//! Aperçus pour l'interface : « que se passerait-il si… ». Lecture seule,
//! aucun effet sur la partie (le moteur reste déterministe).
//!
//! - [`Game::plan_outcome`] : issue d'un plan de population une fois atteint
//!   (travailleurs, production, nourriture mangée, défense) ;
//! - [`Game::estimate_attack`] : cases qu'une attaque prendrait, avec les
//!   mêmes formules que le combat, sans le détail du terrain case par case ;
//! - [`Game::production_split`] : production des bâtiments et du terrain.

use serde::Serialize;

use crate::config::Cost;
use crate::defs::{Effect, Job, Resource, Unit, JOB_COUNT, UNIT_COUNT};
use crate::game::Game;
use crate::player::{PlayerId, NEUTRAL};
use crate::population::{PopulationPlan, MILLI};

/// Issue d'un plan de population (milli, par tour).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct PlanOutcome {
    pub workers: i64,
    pub soldiers: i64,
    pub garrison: i64,
    pub production: Cost,
    pub eaten: i64,
    pub defense: i64,
}

/// Estimation d'une attaque.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct AttackEstimate {
    /// Soldats envoyés (milli).
    pub sent: i64,
    /// Puissance de l'envoi et défense de la cible (milli).
    pub power: i64,
    pub defense: i64,
    /// Cases prises avant que l'attaque ne s'épuise.
    pub tiles: u32,
    /// Vrai si l'attaque prendrait tout le territoire de la cible.
    pub conquers_all: bool,
}

impl Game {
    /// Issue du plan `plan` une fois la population répartie (la garnison garde
    /// son entraînement actuel ; les soldats partis restent partis).
    pub fn plan_outcome(&self, id: PlayerId, plan: &PopulationPlan) -> PlanOutcome {
        let Some(p) = self.player(id) else { return PlanOutcome::default() };
        let population = self.population(id);
        let away = self.away_soldiers(id);
        let away_total: i64 = away.iter().sum();

        // Soldats à la maison par type, comme le fait la formation.
        let unlocked: Vec<bool> = Unit::ALL.iter().map(|&u| self.unit_unlocked(id, u)).collect();
        let mix_total: i64 = (0..UNIT_COUNT).filter(|&u| unlocked[u]).map(|u| plan.unit_mix[u] as i64).sum();
        let target = population * plan.soldiers_permille as i64 / 1000;
        let mut soldiers = [0i64; UNIT_COUNT];
        for u in 0..UNIT_COUNT {
            if unlocked[u] {
                let share = if mix_total > 0 {
                    plan.unit_mix[u] as i64 * 1000 / mix_total
                } else if u == Unit::Melee.index() {
                    1000
                } else {
                    0
                };
                soldiers[u] = (target * share / 1000 - away[u]).max(0);
            }
        }
        let home: i64 = soldiers.iter().sum();
        let garrison = population * plan.garrison_permille as i64 / 1000;
        let workers = (population - home - away_total - garrison).max(0);

        let eaten = self.food_consumption_for(workers, home + away_total, garrison);
        let jobs = self.jobs_for(id, workers, plan.job_mix, eaten);
        let training = if garrison > p.garrison && p.garrison > 0 {
            // Les recrues diluent l'entraînement moyen.
            (p.garrison_training * p.garrison + 1000 * (garrison - p.garrison)) / garrison
        } else if p.garrison > 0 {
            p.garrison_training
        } else {
            1000
        };
        PlanOutcome {
            workers,
            soldiers: home,
            garrison,
            production: self.production_for(id, &jobs),
            eaten,
            defense: self.soldier_defense(id, &soldiers) + self.garrison_defense(id, garrison, training),
        }
    }

    /// Estime une attaque de `attacker` contre `target` avec `ratio_permille`
    /// de ses soldats. `None` sans frontière commune ou sans soldats.
    pub fn estimate_attack(&self, attacker: PlayerId, target: PlayerId, ratio_permille: u16) -> Option<AttackEstimate> {
        let sent = self.take_soldiers_preview(attacker, ratio_permille);
        let sent_total: i64 = sent.iter().sum();
        let mut power = self.attack_power(attacker, &sent);
        if power <= 0 || sent_total < MILLI {
            return None;
        }
        let border = self.border_with(attacker, target);
        if border.is_empty() {
            return None;
        }
        let cfg = &self.config;

        if target == NEUTRAL {
            let cost: i64 =
                border.iter().map(|&t| cfg.terrain_cost[self.map.terrain(t) as usize].max(100)).sum::<i64>() / border.len() as i64;
            let per_tile = (cfg.neutral_tile_power_milli * cost / 1000).max(1);
            return Some(AttackEstimate { sent: sent_total, power, defense: 0, tiles: (power / per_tile) as u32, conquers_all: false });
        }

        // Multiplicateur défensif moyen de la frontière (terrain, rivières, forts).
        let m: i64 = border.iter().map(|&t| self.tile_defense(target, t).0).sum::<i64>() / border.len() as i64;
        let start_defense = self.defense_power(target);
        let mut defense = start_defense;
        let mut left = self.player_ref(target).tiles as i64;
        let chunk = (left / 60).max(1);
        let mut tiles = 0i64;
        while left > 0 {
            let ratio = defense * 1000 / power.max(1);
            let density = defense / left.max(1);
            let loss_ratio = ratio.clamp(cfg.loss_ratio_min, cfg.loss_ratio_max);
            let per_tile = ((cfg.base_tile_power_milli + density * cfg.density_loss_permille / 1000) * m / 1000
                * loss_ratio
                / 1000)
                .max(1);
            let k = chunk.min(left);
            if per_tile * k >= power {
                tiles += power / per_tile;
                break;
            }
            power -= per_tile * k;
            defense = (defense - density * k * cfg.defender_loss_permille / 1000).max(0);
            left -= k;
            tiles += k;
        }
        Some(AttackEstimate {
            sent: sent_total,
            power: self.attack_power(attacker, &sent),
            defense: start_defense,
            tiles: tiles as u32,
            conquers_all: left <= 0,
        })
    }

    /// Production actuelle par tour (milli), séparée entre bâtiments et terrain.
    pub fn production_split(&self, id: PlayerId) -> (Cost, Cost) {
        let Some(p) = self.player(id) else { return (Cost::default(), Cost::default()) };
        let slots = self.job_slots(id);
        let (mut buildings, mut land) = (Cost::default(), Cost::default());
        for job in Job::ALL {
            let j = job.index();
            let in_buildings = p.jobs[j].min(slots.building[j]);
            let from_buildings = if in_buildings > 0 { slots.building_output[j] * in_buildings / slots.building[j] } else { 0 };
            let total = self.production_for(id, &one_job(job, p.jobs[j]));
            let from_buildings = from_buildings * self.experience_factor(id, job, p.jobs[j]) / 1000;
            let b = self.boosted(id, Effect::JobYield(job), from_buildings);
            let amount = resource_of(job.resource(), &total);
            add_to(&mut buildings, job.resource(), b);
            add_to(&mut land, job.resource(), amount - b);
        }
        (buildings, land)
    }
}

fn one_job(job: Job, workers: i64) -> [i64; JOB_COUNT] {
    let mut jobs = [0; JOB_COUNT];
    jobs[job.index()] = workers;
    jobs
}

fn resource_of(r: Resource, c: &Cost) -> i64 {
    match r {
        Resource::Food => c.food,
        Resource::Wood => c.wood,
        Resource::Stone => c.stone,
        Resource::Iron => c.iron,
    }
}

fn add_to(c: &mut Cost, r: Resource, amount: i64) {
    match r {
        Resource::Food => c.food += amount,
        Resource::Wood => c.wood += amount,
        Resource::Stone => c.stone += amount,
        Resource::Iron => c.iron += amount,
    }
}
