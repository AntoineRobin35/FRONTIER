use frontier_engine::rng::Rng;
use frontier_engine::{
    BuildingKind, Deposit, Game, Intent, Phase, PlayerId, PopulationPlan, TechId, TileIdx, Unit, NEUTRAL, NO_REGION,
};

use crate::sense::Surroundings;
use crate::spawn::pick_spawn;
use crate::Difficulty;

/// Briques communes à toutes les IA : placement, rythme de réflexion,
/// population, construction, âges, bateaux. Une IA n'est pas obligée de
/// s'en servir.
pub struct BotCore {
    pub difficulty: Difficulty,
    pub rng: Rng,
    next_think: u64,
}

/// Cible d'un débarquement choisie par [`BotCore::naval_target`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavalGoal {
    /// Seulement des terres libres sur une autre île.
    Settle,
    /// Terres libres de préférence, sinon un joueur faible.
    Any,
}

impl BotCore {
    pub fn new(difficulty: Difficulty, seed: u64) -> Self {
        BotCore {
            difficulty,
            rng: Rng::new(seed),
            next_think: 0,
        }
    }

    /// Gère la phase de placement. Renvoie `true` tant que la partie n'a
    /// pas commencé (l'IA n'a alors rien d'autre à faire).
    pub fn handle_spawn(&mut self, game: &Game, me: PlayerId, out: &mut Vec<Intent>) -> bool {
        if *game.phase() != Phase::Spawn {
            return false;
        }
        let Some(player) = game.player(me) else {
            return true;
        };
        if !player.spawned && game.tick_count() >= me as u64 {
            if let Some(tile) = pick_spawn(game, &mut self.rng) {
                out.push(Intent::Spawn { tile });
            }
        }
        true
    }

    /// Vrai si l'IA doit réfléchir ce tour-ci.
    pub fn should_think(&mut self, game: &Game) -> bool {
        let now = game.tick_count();
        if now < self.next_think {
            return false;
        }
        let base = match self.difficulty {
            Difficulty::Easy => 40,
            Difficulty::Normal => 20,
        };
        self.next_think = now + base + self.rng.below(base as u32 / 2 + 1) as u64;
        !(self.difficulty == Difficulty::Easy && self.rng.chance_permille(250))
    }

    /// Ajuste une part de troupes selon la difficulté (‰).
    pub fn ratio(&self, permille: u16) -> u16 {
        match self.difficulty {
            Difficulty::Easy => permille * 4 / 5,
            Difficulty::Normal => permille,
        }
    }

    /// Demande une répartition de population si elle diffère de l'actuelle.
    pub fn ensure_plan(&self, game: &Game, me: PlayerId, mut plan: PopulationPlan, out: &mut Vec<Intent>) {
        if self.difficulty == Difficulty::Easy {
            plan.soldiers_permille = plan.soldiers_permille * 4 / 5;
        }
        if game.player(me).is_some_and(|p| p.plan != plan) {
            out.push(Intent::SetPlan { plan });
        }
    }

    /// Lance la première recherche possible de la liste (dans l'ordre).
    /// Attend d'avoir les moyens de la recherche prioritaire plutôt que de
    /// dépenser ailleurs.
    pub fn try_research(&self, game: &Game, me: PlayerId, priorities: &[TechId], out: &mut Vec<Intent>) -> bool {
        let Some(p) = game.player(me) else { return false };
        if p.research.is_some() || p.buildings[BuildingKind::Academy.index()] == 0 {
            return false;
        }
        for &tech in priorities {
            if game.can_research(me, tech).is_err() {
                continue;
            }
            let Some((cost, _)) = game.tech_price(me, tech) else { continue };
            if p.resources.covers(&cost) {
                out.push(Intent::Research { tech });
                return true;
            }
            return false;
        }
        false
    }

    pub fn can_afford(game: &Game, me: PlayerId, kind: BuildingKind) -> bool {
        let cost = game.build_cost(me, kind);
        game.player(me).is_some_and(|p| p.resources.covers(&cost))
    }

    /// Construit un bâtiment au meilleur endroit trouvé, si c'est possible.
    pub fn try_build(&mut self, game: &Game, me: PlayerId, kind: BuildingKind, s: &Surroundings, out: &mut Vec<Intent>) -> bool {
        let def = game.config().building(kind);
        if s.tiles.is_empty() || !Self::can_afford(game, me, kind) {
            return false;
        }
        let tile = if def.deposit != Deposit::None {
            s.tiles
                .iter()
                .copied()
                .filter(|&t| game.map().deposit(t) == def.deposit)
                .find(|&t| game.can_build_at(me, t, kind).is_ok())
        } else {
            self.interior_site(game, me, kind, s)
        };
        if let Some(tile) = tile {
            out.push(Intent::Build { tile, kind });
            return true;
        }
        false
    }

    /// Case intérieure (pas sur la frontière), loin des bâtiments du même type.
    fn interior_site(&mut self, game: &Game, me: PlayerId, kind: BuildingKind, s: &Surroundings) -> Option<TileIdx> {
        let mut best: Option<(i64, TileIdx)> = None;
        for _ in 0..150 {
            let tile = s.tiles[self.rng.below(s.tiles.len() as u32) as usize];
            if game.can_build_at(me, tile, kind).is_err() {
                continue;
            }
            if game.map().neighbors(tile).any(|n| game.owner(n) != me) {
                continue;
            }
            let (x, y) = game.map().xy(tile);
            let score = game
                .buildings()
                .iter()
                .filter(|b| b.owner == me && b.kind == kind)
                .map(|b| {
                    let (bx, by) = game.map().xy(b.tile);
                    (bx as i64 - x as i64).pow(2) + (by as i64 - y as i64).pow(2)
                })
                .min()
                .unwrap_or(i64::MAX / 2);
            if best.is_none_or(|(s, _)| score > s) {
                best = Some((score, tile));
            }
        }
        best.map(|(_, t)| t)
    }

    /// Construit un fort un peu en retrait de la frontière avec `enemy`.
    pub fn try_build_fort(&mut self, game: &Game, me: PlayerId, enemy: PlayerId, s: &Surroundings, out: &mut Vec<Intent>) -> bool {
        let kind = BuildingKind::Fort;
        if s.tiles.is_empty() || !Self::can_afford(game, me, kind) {
            return false;
        }
        for _ in 0..300 {
            let tile = s.tiles[self.rng.below(s.tiles.len() as u32) as usize];
            let near_enemy = game
                .map()
                .neighbors(tile)
                .flat_map(|n| game.map().neighbors(n))
                .any(|n| game.owner(n) == enemy);
            if near_enemy && game.can_build_at(me, tile, kind).is_ok() {
                out.push(Intent::Build { tile, kind });
                return true;
            }
        }
        false
    }

    /// Suit une liste de constructions : construit le premier élément dont
    /// on a moins d'exemplaires que voulu.
    pub fn follow_build_order(
        &mut self,
        game: &Game,
        me: PlayerId,
        order: &[(BuildingKind, u32)],
        s: &Surroundings,
        out: &mut Vec<Intent>,
    ) -> bool {
        for &(kind, wanted) in order {
            if game.owned_count(me, kind) >= wanted {
                continue;
            }
            let def = game.config().building(kind);
            if def.max_count.is_some_and(|max| game.owned_count(me, kind) >= max) {
                continue;
            }
            // On attend d'avoir les moyens pour respecter l'ordre…
            if !Self::can_afford(game, me, kind) {
                return false;
            }
            // … mais sans place disponible, on passe au suivant.
            if self.try_build(game, me, kind, s, out) {
                return true;
            }
        }
        false
    }

    /// Puissance de l'attaque qu'on enverrait avec cette part des soldats.
    pub fn strike_power(game: &Game, me: PlayerId, ratio: u16) -> i64 {
        let Some(p) = game.player(me) else { return 0 };
        let mut units = p.soldiers;
        for u in units.iter_mut() {
            *u = *u * ratio as i64 / 1000;
        }
        game.attack_power(me, &units)
    }

    /// Mélange d'unités souhaité, limité aux unités débloquées.
    pub fn unit_mix(game: &Game, me: PlayerId, wanted: [u16; 3]) -> [u16; 3] {
        let mut mix = [0u16; 3];
        for u in Unit::ALL {
            if game.unit_unlocked(me, u) {
                mix[u.index()] = wanted[u.index()];
            }
        }
        if mix.iter().all(|&m| m == 0) {
            mix[Unit::Melee.index()] = 1000;
        }
        mix
    }

    /// Choisit une côte à atteindre par bateau, sur une terre où je ne suis
    /// pas encore : de préférence des terres libres, proches de chez moi.
    pub fn naval_target(&mut self, game: &Game, me: PlayerId, s: &Surroundings, goal: NavalGoal) -> Option<TileIdx> {
        let map = game.map();
        let regions = game.regions();
        let home = game.player(me)?.spawn_tile.map(|t| map.xy(t))?;
        let my_power = Self::strike_power(game, me, 500);

        let mut best: Option<(i64, TileIdx)> = None;
        for _ in 0..600 {
            let tile = self.rng.below(map.len() as u32);
            if !map.is_passable(tile) || !game.is_coastal(tile) {
                continue;
            }
            let region = regions.id[tile as usize];
            if region == NO_REGION || s.my_regions[region as usize] {
                continue;
            }
            let owner = game.owner(tile);
            let mut score = if owner == NEUTRAL {
                1_000_000
            } else if goal == NavalGoal::Any && game.defense_power(owner) * 2 < my_power {
                0
            } else {
                continue;
            };
            let (x, y) = map.xy(tile);
            score -= (x as i64 - home.0 as i64).pow(2) + (y as i64 - home.1 as i64).pow(2);
            if best.is_none_or(|(s, _)| score > s) {
                best = Some((score, tile));
            }
        }
        best.map(|(_, t)| t)
    }
}
