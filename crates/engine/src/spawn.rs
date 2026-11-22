//! Phase de placement : chaque joueur choisit son point de départ.

use crate::defs::Unit;
use crate::population::{PopulationPlan, MILLI};
use crate::game::{Game, Phase};
use crate::map::TileIdx;
use crate::player::{PlayerId, NEUTRAL};
use crate::update::{GameEvent, RejectReason};

impl Game {
    pub(crate) fn spawn(&mut self, player: PlayerId, tile: TileIdx) -> Result<(), RejectReason> {
        if self.phase != Phase::Spawn {
            return Err(RejectReason::WrongPhase);
        }
        if self.player_ref(player).spawned {
            return Err(RejectReason::AlreadySpawned);
        }
        if tile as usize >= self.map.len() || !self.map.is_passable(tile) {
            return Err(RejectReason::InvalidTile);
        }
        if self.owner(tile) != NEUTRAL {
            return Err(RejectReason::TileTaken);
        }

        self.claim_start_area(player, tile);
        let cfg = &self.config;
        let population = cfg.start_population * MILLI;
        let soldiers = population * cfg.start_soldiers_permille as i64 / 1000;
        let resources = cfg.start_resources.to_milli();
        let plan = PopulationPlan {
            soldiers_permille: cfg.start_soldiers_permille,
            ..PopulationPlan::default()
        };
        let p = self.player_mut(player);
        p.spawned = true;
        p.spawn_tile = Some(tile);
        p.resources = resources;
        p.workers = population - soldiers;
        p.soldiers[Unit::Melee.index()] = soldiers;
        p.plan = plan;
        self.update.events.push(GameEvent::Spawned { player, tile });
        Ok(())
    }

    /// Prend les cases neutres reliées au point de départ, dans un disque.
    fn claim_start_area(&mut self, player: PlayerId, start: TileIdx) {
        let r = self.config.spawn_radius as i64;
        let (cx, cy) = self.map.xy(start);
        let mut stack = vec![start];
        self.set_owner(start, player);
        while let Some(tile) = stack.pop() {
            for next in self.map.neighbors(tile) {
                if self.owner(next) != NEUTRAL || !self.map.is_passable(next) {
                    continue;
                }
                let (x, y) = self.map.xy(next);
                let dx = x as i64 - cx as i64;
                let dy = y as i64 - cy as i64;
                if dx * dx + dy * dy <= r * r {
                    self.set_owner(next, player);
                    stack.push(next);
                }
            }
        }
    }

    pub(crate) fn update_spawn_phase(&mut self) {
        let timed_out = self
            .config
            .spawn_phase_max_ticks
            .is_some_and(|max| self.tick + 1 >= max as u64);

        if timed_out {
            let missing: Vec<PlayerId> =
                self.players.iter().filter(|p| !p.spawned).map(|p| p.id).collect();
            for player in missing {
                if let Some(tile) = self.random_free_tile() {
                    // Ne peut pas échouer : la case vient d'être validée.
                    let _ = self.spawn(player, tile);
                }
            }
        }

        if timed_out || self.players.iter().all(|p| p.spawned) {
            for p in &mut self.players {
                if !p.spawned {
                    p.alive = false;
                }
            }
            self.phase = Phase::Playing;
            self.started_at = Some(self.tick);
            self.update.events.push(GameEvent::GameStarted);
        }
    }

    fn random_free_tile(&mut self) -> Option<TileIdx> {
        let n = self.map.len() as u32;
        for _ in 0..10_000 {
            let tile = self.rng.below(n);
            if self.map.is_passable(tile) && self.owner(tile) == NEUTRAL {
                return Some(tile);
            }
        }
        None
    }
}
