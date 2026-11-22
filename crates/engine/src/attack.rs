//! Attaques terrestres : une armée (guerriers, archers, cavaliers) grignote
//! la frontière d'une cible case par case.
//!
//! Principe (inspiré d'OpenFront) : à chaque case, on compare la défense
//! totale du défenseur à la puissance de l'attaque. Face à un défenseur
//! faible, le front avance vite et perd peu ; face à un défenseur fort, il
//! ralentit et fond. Le terrain, les rivières et les forts renforcent la
//! défense ; chaque type d'unité a ses terrains favoris.
//!
//! Les effectifs des attaques sont en milli-soldats ; les puissances en
//! milli (1000 = un guerrier de l'âge du Bois).

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::defs::{BuildingKind, Effect, Unit, UNIT_COUNT};
use crate::game::{Game, Phase};
use crate::map::TileIdx;
use crate::player::{PlayerId, NEUTRAL};
use crate::population::remove_proportional;
use crate::rng::hash2;
use crate::update::{GameEvent, RejectReason};

#[derive(Clone, Debug)]
pub struct Attack {
    pub id: u32,
    pub attacker: PlayerId,
    pub target: PlayerId,
    /// Soldats engagés (milli), par type.
    pub units: [i64; UNIT_COUNT],
    /// Avancée accumulée (‰ de case) : permet aux fronts lents de prendre
    /// une case tous les quelques tours.
    pub(crate) progress: i64,
    /// Cases à prendre, les plus proches et les moins chères d'abord.
    pub(crate) frontier: BinaryHeap<Reverse<FrontierEntry>>,
}

impl Attack {
    /// Nombre de soldats (milli).
    pub fn soldiers(&self) -> i64 {
        self.units.iter().sum()
    }
}

/// Entrée du front. L'ordre (priorité, puis case) est total et déterministe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FrontierEntry {
    pub priority: u32,
    pub tile: TileIdx,
    /// Case de départ de cette avancée : la priorité suit la distance réelle
    /// à cette case, ce qui donne des fronts arrondis.
    pub source: TileIdx,
}

impl Game {
    // ------------------------------------------------------------------
    // Puissances
    // ------------------------------------------------------------------

    /// Puissance d'attaque brute d'une armée (milli), sans terrain.
    pub fn attack_power(&self, owner: PlayerId, units: &[i64; UNIT_COUNT]) -> i64 {
        Unit::ALL
            .iter()
            .map(|&u| {
                let base = units[u.index()] * self.config.unit(u).attack_permille / 1000;
                self.boosted(owner, Effect::UnitAttack(u), base)
            })
            .sum()
    }

    /// Puissance d'attaque sur une case précise (terrain, fort).
    fn attack_power_on(&self, owner: PlayerId, units: &[i64; UNIT_COUNT], tile: TileIdx, fort: bool) -> i64 {
        let open = self.config.terrain_open[self.map.terrain(tile) as usize];
        Unit::ALL
            .iter()
            .map(|&u| {
                let def = self.config.unit(u);
                let terrain = if open { def.open_permille } else { def.rough_permille };
                let siege = if fort { def.siege_permille } else { 1000 };
                let base = units[u.index()] * def.attack_permille / 1000 * terrain / 1000 * siege / 1000;
                self.boosted(owner, Effect::UnitAttack(u), base)
            })
            .sum()
    }

    /// Défense totale d'un joueur (milli) : soldats à la maison et garnison.
    pub fn defense_power(&self, id: PlayerId) -> i64 {
        let Some(p) = self.player(id) else { return 0 };
        let soldiers: i64 = Unit::ALL
            .iter()
            .map(|&u| p.soldiers[u.index()] * self.config.unit(u).defense_permille / 1000)
            .sum();
        let garrison = p.garrison * p.garrison_training / 1000;
        self.boosted(id, Effect::SoldierDefense, soldiers) + self.boosted(id, Effect::GarrisonDefense, garrison)
    }

    /// Multiplicateur défensif d'une case (‰) : terrain, rivière, fort.
    pub fn tile_defense(&self, owner: PlayerId, tile: TileIdx) -> (i64, bool) {
        let cfg = &self.config;
        let mut m = cfg.terrain_defense[self.map.terrain(tile) as usize].max(100);
        if self.map.has_river(tile) {
            m = m * cfg.river_defense_permille / 1000;
        }
        let fort = owner != NEUTRAL && self.protected_by_fort(owner, tile);
        if fort {
            m = m * self.boosted(owner, Effect::FortDefense, cfg.fort_defense_permille) / 1000;
        }
        (m, fort)
    }

    /// Vitesse de l'armée (‰), moyenne pondérée des types.
    fn army_speed(&self, units: &[i64; UNIT_COUNT]) -> i64 {
        let total: i64 = units.iter().sum();
        if total == 0 {
            return 1000;
        }
        Unit::ALL
            .iter()
            .map(|&u| units[u.index()] * self.config.unit(u).speed_permille)
            .sum::<i64>()
            / total
    }

    // ------------------------------------------------------------------
    // Ordres
    // ------------------------------------------------------------------

    pub(crate) fn immune(&self) -> bool {
        self.started_at
            .is_none_or(|start| self.tick < start + self.config.immunity_ticks as u64)
    }

    pub(crate) fn launch_attack(
        &mut self,
        attacker: PlayerId,
        target: PlayerId,
        ratio_permille: u16,
    ) -> Result<(), RejectReason> {
        if self.phase != Phase::Playing {
            return Err(RejectReason::WrongPhase);
        }
        if target == attacker {
            return Err(RejectReason::InvalidTarget);
        }
        if target != NEUTRAL {
            if !self.player(target).is_some_and(|p| p.alive) {
                return Err(RejectReason::InvalidTarget);
            }
            if self.immune() {
                return Err(RejectReason::Immune);
            }
        }

        let mut units = self.take_soldiers_preview(attacker, ratio_permille);
        if units.iter().sum::<i64>() < self.config.min_attack_soldiers * 1000 {
            return Err(RejectReason::NotEnoughTroops);
        }
        let border = self.border_with(attacker, target);
        if border.is_empty() {
            return Err(RejectReason::NoBorder);
        }
        self.take_soldiers(attacker, &units);

        // Contre-attaque : les deux armées se rencontrent.
        if let Some(j) = self
            .attacks
            .iter()
            .position(|a| a.attacker == target && a.target == attacker)
        {
            let mine = self.attack_power(attacker, &units);
            let theirs = self.attack_power(target, &self.attacks[j].units);
            let clash = mine.min(theirs);
            if mine > 0 {
                let n = units.iter().sum::<i64>() * clash / mine;
                remove_proportional(&mut units, n);
            }
            if theirs > 0 {
                let other = &mut self.attacks[j].units;
                let n = other.iter().sum::<i64>() * clash / theirs;
                remove_proportional(other, n);
            }
            self.update.events.push(GameEvent::AttacksClashed {
                a: attacker,
                b: target,
                troops: clash / 1000,
            });
            if self.attacks[j].soldiers() < 1000 {
                let ended = self.attacks.remove(j);
                self.finish_attack(ended, 1000);
            }
            if units.iter().sum::<i64>() < 1000 {
                return Ok(());
            }
        }

        self.start_attack(attacker, target, units, border);
        Ok(())
    }

    /// Soldats qu'enverrait une attaque de cette proportion (milli).
    pub(crate) fn take_soldiers_preview(&self, id: PlayerId, ratio_permille: u16) -> [i64; UNIT_COUNT] {
        let p = self.player_ref(id);
        let ratio = ratio_permille.min(1000) as i64;
        let mut units = [0; UNIT_COUNT];
        for u in 0..UNIT_COUNT {
            units[u] = p.soldiers[u] * ratio / 1000;
        }
        units
    }

    pub(crate) fn take_soldiers(&mut self, id: PlayerId, units: &[i64; UNIT_COUNT]) {
        let p = self.player_mut(id);
        for u in 0..UNIT_COUNT {
            p.soldiers[u] -= units[u];
        }
    }

    /// Crée une attaque (ou renforce celle qui existe contre la même cible).
    /// Les soldats doivent déjà avoir été retirés au joueur.
    pub(crate) fn start_attack(
        &mut self,
        attacker: PlayerId,
        target: PlayerId,
        units: [i64; UNIT_COUNT],
        seeds: Vec<TileIdx>,
    ) {
        let entries = seeds.into_iter().map(|tile| {
            Reverse(FrontierEntry {
                priority: 0,
                tile,
                source: tile,
            })
        });
        let troops = units.iter().sum::<i64>() / 1000;

        if let Some(existing) = self
            .attacks
            .iter_mut()
            .find(|a| a.attacker == attacker && a.target == target)
        {
            for u in 0..UNIT_COUNT {
                existing.units[u] += units[u];
            }
            existing.frontier.extend(entries);
            let attack_id = existing.id;
            self.update.events.push(GameEvent::AttackReinforced {
                attack_id,
                attacker,
                target,
                troops,
            });
            return;
        }

        let id = self.next_id();
        let mut frontier = BinaryHeap::new();
        frontier.extend(entries);
        self.attacks.push(Attack {
            id,
            attacker,
            target,
            units,
            progress: 0,
            frontier,
        });
        self.update.events.push(GameEvent::AttackStarted {
            attack_id: id,
            attacker,
            target,
            troops,
        });
    }

    /// Cases de `target` qui touchent le territoire de `attacker`.
    pub(crate) fn border_with(&self, attacker: PlayerId, target: PlayerId) -> Vec<TileIdx> {
        let mut border = Vec::new();
        for tile in 0..self.map.len() as TileIdx {
            if self.owner[tile as usize] != attacker {
                continue;
            }
            for n in self.map.neighbors(tile) {
                if self.owner[n as usize] == target && self.map.is_passable(n) {
                    border.push(n);
                }
            }
        }
        border.sort_unstable();
        border.dedup();
        border
    }

    pub(crate) fn cancel_attack(&mut self, player: PlayerId, attack_id: u32) -> Result<(), RejectReason> {
        let Some(j) = self
            .attacks
            .iter()
            .position(|a| a.id == attack_id && a.attacker == player)
        else {
            return Err(RejectReason::UnknownAttack);
        };
        let attack = self.attacks.remove(j);
        // Battre en retraite face à un joueur coûte des hommes.
        let kept = if attack.target == NEUTRAL {
            1000
        } else {
            1000 - self.boosted(player, Effect::RetreatLoss, self.config.retreat_loss_permille)
        };
        self.finish_attack(attack, kept);
        Ok(())
    }

    /// Termine une attaque : `kept_permille` des soldats rentrent.
    pub(crate) fn finish_attack(&mut self, attack: Attack, kept_permille: i64) {
        let alive = self.player_ref(attack.attacker).alive;
        let mut returned = 0;
        if alive {
            let p = self.player_mut(attack.attacker);
            for u in 0..UNIT_COUNT {
                let n = attack.units[u] * kept_permille / 1000;
                p.soldiers[u] += n;
                returned += n;
            }
        }
        self.update.events.push(GameEvent::AttackEnded {
            attack_id: attack.id,
            attacker: attack.attacker,
            target: attack.target,
            returned: returned / 1000,
        });
    }

    // ------------------------------------------------------------------
    // Tour
    // ------------------------------------------------------------------

    pub(crate) fn run_attacks(&mut self) {
        let mut attacks = std::mem::take(&mut self.attacks);
        let mut still_running = Vec::with_capacity(attacks.len());
        for mut attack in attacks.drain(..) {
            if self.advance_attack(&mut attack) {
                still_running.push(attack);
            } else {
                self.finish_attack(attack, 1000);
            }
        }
        still_running.append(&mut self.attacks);
        self.attacks = still_running;
    }

    /// Fait avancer une attaque d'un tour. Renvoie `false` si elle est finie.
    fn advance_attack(&mut self, attack: &mut Attack) -> bool {
        if !self.player_ref(attack.attacker).alive {
            return false;
        }
        if attack.target != NEUTRAL && !self.player_ref(attack.target).alive {
            return false;
        }
        let cfg = self.config.clone();
        let power = self.attack_power(attack.attacker, &attack.units);
        if power <= 0 {
            return false;
        }

        // Rapport de forces (‰) et vitesse de base.
        let (ratio, speed) = if attack.target == NEUTRAL {
            let speed = (power * 1000 / cfg.neutral_speed_scale.max(1))
                .clamp(cfg.neutral_speed_min, cfg.neutral_speed_max);
            (1000, self.boosted(attack.attacker, Effect::NeutralSpeed, speed))
        } else {
            let defense = self.defense_power(attack.target);
            (defense * 1000 / power, cfg.attack_speed_permille)
        };
        let speed = speed * self.army_speed(&attack.units) / 1000;
        // Réserve d'avancée : assez pour franchir la case la plus lente.
        attack.progress = (attack.progress + speed).min(speed * 4 + 20_000);

        let mut taken = 0u32;
        while taken < cfg.max_tiles_per_tick {
            let Some(&Reverse(entry)) = attack.frontier.peek() else {
                return false; // plus rien à prendre
            };
            let tile = entry.tile;
            if self.owner(tile) != attack.target
                || !self.map.is_passable(tile)
                || !self.touches(tile, attack.attacker)
            {
                attack.frontier.pop();
                continue; // entrée périmée
            }

            let (tile_time, loss) = self.tile_price(attack, tile, ratio);
            if attack.progress < tile_time {
                break; // la suite au prochain tour
            }
            let tile_power = self.attack_power(attack.attacker, &attack.units);
            if loss >= tile_power {
                return false; // l'attaque s'essouffle
            }
            attack.frontier.pop();
            attack.progress -= tile_time;
            taken += 1;

            // Pertes de l'attaquant, réparties entre les types.
            let soldiers = attack.soldiers();
            remove_proportional(&mut attack.units, soldiers * loss / tile_power.max(1));

            // Pertes du défenseur.
            if attack.target != NEUTRAL {
                self.defender_loses(attack.target);
            }

            self.set_owner(tile, attack.attacker);
            for next in self.map.neighbors(tile) {
                if self.owner(next) == attack.target && self.map.is_passable(next) {
                    attack.frontier.push(Reverse(FrontierEntry {
                        priority: self.frontier_priority(next, entry.source, attack.id),
                        tile: next,
                        source: entry.source,
                    }));
                }
            }
        }
        true
    }

    /// Temps (‰ de case) et pertes (milli de puissance) pour prendre une case.
    fn tile_price(&self, attack: &Attack, tile: TileIdx, ratio: i64) -> (i64, i64) {
        let cfg = &self.config;
        if attack.target == NEUTRAL {
            let cost = cfg.terrain_cost[self.map.terrain(tile) as usize].max(100);
            return (cost, cfg.neutral_tile_power_milli * cost / 1000);
        }
        let (m, fort) = self.tile_defense(attack.target, tile);
        let target = self.player_ref(attack.target);
        let density = self.defense_power(attack.target) / target.tiles.max(1) as i64;
        let loss_ratio = ratio.clamp(cfg.loss_ratio_min, cfg.loss_ratio_max);
        let speed_ratio = ratio.clamp(cfg.speed_ratio_min, cfg.speed_ratio_max);

        // Les unités à l'aise sur ce terrain perdent moins.
        let raw = self.attack_power(attack.attacker, &attack.units).max(1);
        let here = self.attack_power_on(attack.attacker, &attack.units, tile, fort).max(1);
        let loss = (cfg.base_tile_power_milli + density) * m / 1000 * loss_ratio / 1000 * raw / here;
        let time = m * speed_ratio / 1000;
        (time, loss)
    }

    /// Le défenseur perd une part de sa densité, répartie entre soldats et garnison.
    fn defender_loses(&mut self, id: PlayerId) {
        let loss_permille = self.config.defender_loss_permille;
        let p = self.player_mut(id);
        let tiles = p.tiles.max(1) as i64;
        let army = p.army();
        remove_proportional(&mut p.soldiers, army * loss_permille / 1000 / tiles);
        let garrison_loss = p.garrison * loss_permille / 1000 / tiles;
        p.garrison = (p.garrison - garrison_loss).max(0);
    }

    /// Ordre de conquête (plus petit = pris d'abord). Le front avance selon
    /// la distance réelle depuis sa case de départ, contourne un peu les
    /// terrains difficiles, avec un bruit doux qui le fait avancer par poches.
    fn frontier_priority(&self, tile: TileIdx, source: TileIdx, attack_id: u32) -> u32 {
        let (x, y) = self.map.xy(tile);
        let (sx, sy) = self.map.xy(source);
        let dx = (x as i64 - sx as i64).unsigned_abs();
        let dy = (y as i64 - sy as i64).unsigned_abs();
        let distance = ((dx * dx + dy * dy) * 1_000_000).isqrt() as u32;

        let terrain = self.config.terrain_cost[self.map.terrain(tile) as usize] as u32;
        let salt = hash2(self.map.seed, attack_id as u64);
        let pockets = smooth_noise(x, y, 14, salt) * 3 / 2 + smooth_noise(x, y, 5, salt ^ 1) / 2;
        let grain = (hash2(tile as u64, salt) % 200) as u32;
        distance + terrain / 2 + pockets + grain
    }

    /// Vrai si un fort terminé du joueur protège cette case.
    fn protected_by_fort(&self, owner: PlayerId, tile: TileIdx) -> bool {
        if self.player_ref(owner).buildings[BuildingKind::Fort.index()] == 0 {
            return false;
        }
        let (x, y) = self.map.xy(tile);
        let r = self.config.fort_radius as i64;
        self.buildings.iter().any(|b| {
            if b.owner != owner || b.kind != BuildingKind::Fort || b.remaining > 0 {
                return false;
            }
            let (bx, by) = self.map.xy(b.tile);
            let dx = bx as i64 - x as i64;
            let dy = by as i64 - y as i64;
            dx * dx + dy * dy <= r * r
        })
    }

    /// Prix d'un débarquement sur une case (pertes en milli de puissance).
    pub(crate) fn landing_loss(&self, attacker: PlayerId, units: &[i64; UNIT_COUNT], tile: TileIdx) -> i64 {
        let defender = self.owner(tile);
        let probe = Attack {
            id: 0,
            attacker,
            target: defender,
            units: *units,
            progress: 0,
            frontier: BinaryHeap::new(),
        };
        let ratio = if defender == NEUTRAL {
            1000
        } else {
            self.defense_power(defender) * 1000 / self.attack_power(attacker, units).max(1)
        };
        self.tile_price(&probe, tile, ratio).1
    }
}

/// Bruit de valeur entier, lissé sur des cellules de `cell` cases.
/// Renvoie une valeur dans `0..1000`. Entièrement déterministe.
fn smooth_noise(x: u32, y: u32, cell: u32, salt: u64) -> u32 {
    let (cx, cy) = (x / cell, y / cell);
    let (fx, fy) = (x % cell, y % cell);
    let corner =
        |i: u32, j: u32| (hash2(salt, ((cx + i) as u64) << 32 | (cy + j) as u64) % 1000) as u32;
    let top = corner(0, 0) * (cell - fx) + corner(1, 0) * fx;
    let bottom = corner(0, 1) * (cell - fx) + corner(1, 1) * fx;
    (top * (cell - fy) + bottom * fy) / (cell * cell)
}
