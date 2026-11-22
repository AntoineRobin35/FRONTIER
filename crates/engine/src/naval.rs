//! Bateaux : envoyer une armée par la mer vers une côte qu'on ne peut pas
//! atteindre à pied (autre île, autre continent, arrière d'un ennemi).
//!
//! Le bateau suit le plus court chemin sur l'eau, puis débarque : il prend la
//! case d'arrivée et lance une attaque depuis cette tête de pont.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::defs::UNIT_COUNT;
use crate::game::{Game, Phase};
use crate::map::TileIdx;
use crate::player::{PlayerId, NEUTRAL};
use crate::population::{remove_proportional, MILLI};
use crate::update::{GameEvent, RejectReason};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Boat {
    pub id: u32,
    pub owner: PlayerId,
    /// Soldats à bord (milli), par type.
    pub units: [i64; UNIT_COUNT],
    /// Chemin sur l'eau, terminé par la case de débarquement (terre).
    pub(crate) path: Vec<TileIdx>,
    pub(crate) progress: usize,
}

impl Boat {
    pub fn tile(&self) -> TileIdx {
        self.path[self.progress.min(self.path.len() - 1)]
    }

    pub fn destination(&self) -> TileIdx {
        *self.path.last().expect("chemin non vide")
    }

    pub fn soldiers(&self) -> i64 {
        self.units.iter().sum()
    }
}

impl Game {
    pub(crate) fn launch_boat(
        &mut self,
        player: PlayerId,
        target: TileIdx,
        ratio_permille: u16,
    ) -> Result<(), RejectReason> {
        if self.phase != Phase::Playing {
            return Err(RejectReason::WrongPhase);
        }
        if target as usize >= self.map.len() || !self.map.is_passable(target) || !self.is_coastal(target) {
            return Err(RejectReason::InvalidTile);
        }
        let defender = self.owner(target);
        if defender == player {
            return Err(RejectReason::InvalidTarget);
        }
        if defender != NEUTRAL && self.immune() {
            return Err(RejectReason::Immune);
        }
        if self.boats.iter().filter(|b| b.owner == player).count() >= self.config.max_boats as usize {
            return Err(RejectReason::TooManyBoats);
        }
        let wood = self.config.boat_wood * MILLI;
        if self.player_ref(player).resources.wood < wood {
            return Err(RejectReason::NotEnoughResources);
        }
        let units = self.take_soldiers_preview(player, ratio_permille);
        if units.iter().sum::<i64>() < self.config.min_attack_soldiers * MILLI {
            return Err(RejectReason::NotEnoughTroops);
        }
        let Some(path) = self.sea_path(player, target) else {
            return Err(RejectReason::NoPath);
        };

        self.take_soldiers(player, &units);
        self.player_mut(player).resources.wood -= wood;
        let id = self.next_id();
        let troops = units.iter().sum::<i64>() / MILLI;
        self.boats.push(Boat {
            id,
            owner: player,
            units,
            path,
            progress: 0,
        });
        self.update.events.push(GameEvent::BoatLaunched {
            boat_id: id,
            owner: player,
            tile: target,
            troops,
        });
        Ok(())
    }

    /// Plus court chemin sur l'eau entre la côte du joueur et la case cible.
    fn sea_path(&self, player: PlayerId, target: TileIdx) -> Option<Vec<TileIdx>> {
        let n = self.map.len();
        const UNSEEN: u32 = u32::MAX;
        let mut parent = vec![UNSEEN; n];
        let mut queue = VecDeque::new();

        for tile in 0..n as TileIdx {
            if self.owner[tile as usize] != player {
                continue;
            }
            for w in self.map.neighbors(tile) {
                if self.map.terrain(w).is_water() && parent[w as usize] == UNSEEN {
                    parent[w as usize] = w;
                    queue.push_back(w);
                }
            }
        }

        while let Some(current) = queue.pop_front() {
            if self.map.neighbors(current).any(|t| t == target) {
                let mut path = vec![target];
                let mut c = current;
                loop {
                    path.push(c);
                    let p = parent[c as usize];
                    if p == c {
                        break;
                    }
                    c = p;
                }
                path.reverse();
                return Some(path);
            }
            for next in self.map.neighbors(current) {
                if parent[next as usize] == UNSEEN && self.map.terrain(next).is_water() {
                    parent[next as usize] = current;
                    queue.push_back(next);
                }
            }
        }
        None
    }

    pub(crate) fn run_boats(&mut self) {
        let speed = self.config.boat_speed.max(1) as usize;
        let boats = std::mem::take(&mut self.boats);
        let mut sailing = Vec::with_capacity(boats.len());
        for mut boat in boats {
            boat.progress += speed;
            if boat.progress + 1 >= boat.path.len() {
                self.land(boat);
            } else {
                sailing.push(boat);
            }
        }
        sailing.append(&mut self.boats);
        self.boats = sailing;
    }

    fn land(&mut self, mut boat: Boat) {
        let tile = boat.destination();
        let owner = boat.owner;
        let defender = self.owner(tile);

        if defender == owner {
            let p = self.player_mut(owner);
            for u in 0..UNIT_COUNT {
                p.soldiers[u] += boat.units[u];
            }
            self.update.events.push(GameEvent::BoatLanded { boat_id: boat.id, owner, tile });
            return;
        }

        let power = self.attack_power(owner, &boat.units);
        let loss = self.landing_loss(owner, &boat.units, tile);
        if power <= loss {
            self.update.events.push(GameEvent::BoatLost { boat_id: boat.id, owner });
            return;
        }
        let soldiers = boat.soldiers();
        remove_proportional(&mut boat.units, soldiers * loss / power.max(1));
        self.set_owner(tile, owner);
        self.update.events.push(GameEvent::BoatLanded { boat_id: boat.id, owner, tile });

        let seeds: Vec<TileIdx> = self
            .map
            .neighbors(tile)
            .filter(|&n| self.owner(n) == defender && self.map.is_passable(n))
            .collect();
        if seeds.is_empty() {
            let p = self.player_mut(owner);
            for u in 0..UNIT_COUNT {
                p.soldiers[u] += boat.units[u];
            }
        } else {
            self.start_attack(owner, defender, boat.units, seeds);
        }
    }
}
