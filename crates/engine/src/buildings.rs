//! Bâtiments : chantier de quelques dizaines de secondes, coût croissant,
//! âge requis, gisement requis pour les bâtiments de production. Un bâtiment
//! change de camp (même en chantier) quand sa case est prise.

use serde::{Deserialize, Serialize};

use crate::config::Cost;
use crate::defs::{BuildingKind, Deposit, Effect};
use crate::game::{Game, Phase, NO_BUILDING};
use crate::map::TileIdx;
use crate::player::{PlayerId, NEUTRAL};
use crate::update::{GameEvent, RejectReason};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Building {
    pub id: u32,
    pub kind: BuildingKind,
    pub tile: TileIdx,
    pub owner: PlayerId,
    /// Tours de chantier restants (0 = terminé).
    pub remaining: u32,
}

impl Game {
    /// Bâtiments d'un type possédés par un joueur (chantiers compris).
    pub fn owned_count(&self, id: PlayerId, kind: BuildingKind) -> u32 {
        self.buildings.iter().filter(|b| b.owner == id && b.kind == kind).count() as u32
    }

    /// Coût du prochain bâtiment de ce type (milli). Il augmente avec le
    /// nombre déjà possédé, chantiers compris.
    pub fn build_cost(&self, id: PlayerId, kind: BuildingKind) -> Cost {
        let def = self.config.building(kind);
        let mut cost = def.cost.to_milli();
        for _ in 0..self.owned_count(id, kind).min(40) {
            cost = cost.scaled(100 + def.cost_growth_percent as i64, 100);
        }
        cost
    }

    /// Vérifie qu'un bâtiment peut être placé ici (sans regarder le coût).
    pub fn can_build_at(&self, id: PlayerId, tile: TileIdx, kind: BuildingKind) -> Result<(), RejectReason> {
        let def = self.config.building(kind);
        if self.player(id).is_none() {
            return Err(RejectReason::UnknownPlayer);
        }
        if def.max_count.is_some_and(|max| self.owned_count(id, kind) >= max) {
            return Err(RejectReason::MaxCount);
        }
        if tile as usize >= self.map.len() || !self.map.is_passable(tile) {
            return Err(RejectReason::InvalidTile);
        }
        if self.owner(tile) != id {
            return Err(RejectReason::NotOwned);
        }
        if self.building_at[tile as usize] != NO_BUILDING {
            return Err(RejectReason::TileTaken);
        }
        if def.deposit != Deposit::None && self.map.deposit(tile) != def.deposit {
            return Err(RejectReason::NeedsDeposit);
        }
        let min = def.min_distance as i64;
        let (x, y) = self.map.xy(tile);
        let too_close = self.buildings.iter().any(|b| {
            if b.kind != kind {
                return false;
            }
            let (bx, by) = self.map.xy(b.tile);
            let dx = bx as i64 - x as i64;
            let dy = by as i64 - y as i64;
            dx * dx + dy * dy < min * min
        });
        if too_close {
            return Err(RejectReason::TooClose);
        }
        Ok(())
    }

    pub(crate) fn build(&mut self, id: PlayerId, tile: TileIdx, kind: BuildingKind) -> Result<(), RejectReason> {
        if self.phase != Phase::Playing {
            return Err(RejectReason::WrongPhase);
        }
        self.can_build_at(id, tile, kind)?;
        let cost = self.build_cost(id, kind);
        if !self.player_ref(id).resources.covers(&cost) {
            return Err(RejectReason::NotEnoughResources);
        }

        let building_id = self.next_id();
        // L'architecture (Académie) raccourcit les chantiers.
        let base = self.config.building(kind).build_ticks as i64;
        let remaining = (base * 1000 / self.modifier(id, Effect::BuildSpeed).max(1)).max(1) as u32;
        self.player_mut(id).resources.pay(&cost);
        self.building_at[tile as usize] = self.buildings.len() as u32;
        self.buildings.push(Building {
            id: building_id,
            kind,
            tile,
            owner: id,
            remaining,
        });
        self.update.events.push(GameEvent::BuildStarted {
            building_id,
            owner: id,
            kind,
            tile,
        });
        Ok(())
    }

    /// Fait avancer les chantiers.
    pub(crate) fn run_buildings(&mut self) {
        for i in 0..self.buildings.len() {
            // Terminé, ou ruine (sans propriétaire) : rien à faire.
            if self.buildings[i].remaining == 0 || self.buildings[i].owner == NEUTRAL {
                continue;
            }
            self.buildings[i].remaining -= 1;
            if self.buildings[i].remaining == 0 {
                let b = self.buildings[i].clone();
                self.player_mut(b.owner).buildings[b.kind.index()] += 1;
                self.update.events.push(GameEvent::BuildingCompleted {
                    building_id: b.id,
                    owner: b.owner,
                    kind: b.kind,
                    tile: b.tile,
                });
            }
        }
    }

    /// Appelé par `set_owner` quand une case avec un bâtiment change de main.
    pub(crate) fn capture_building(&mut self, tile: TileIdx, from: PlayerId, to: PlayerId) {
        let index = self.building_at[tile as usize] as usize;
        let (kind, building_id, done, ruin) = {
            let b = &self.buildings[index];
            (b.kind, b.id, b.remaining == 0, b.owner == NEUTRAL)
        };
        if ruin {
            return;
        }
        // Un fort pris est détruit : il reste une ruine qui ne protège personne.
        if kind == BuildingKind::Fort {
            if done && from != NEUTRAL {
                let c = &mut self.player_mut(from).buildings[kind.index()];
                *c = c.saturating_sub(1);
            }
            self.buildings[index].owner = NEUTRAL;
            self.update.events.push(GameEvent::BuildingDestroyed { building_id, kind, owner: from });
            return;
        }
        self.buildings[index].owner = to;
        if done {
            if from != NEUTRAL {
                let c = &mut self.player_mut(from).buildings[kind.index()];
                *c = c.saturating_sub(1);
            }
            if to != NEUTRAL {
                self.player_mut(to).buildings[kind.index()] += 1;
            }
        }
        self.update.events.push(GameEvent::BuildingCaptured {
            building_id,
            kind,
            from,
            to,
        });
    }
}
