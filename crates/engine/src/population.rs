//! Population et économie : naissances nourries par la nourriture, métiers
//! des travailleurs, formation des soldats et de la garnison, passage des âges.
//!
//! Tout est compté en millièmes (`MILLI` = une personne ou une unité).

use serde::{Deserialize, Serialize};

use crate::config::Cost;
use crate::defs::{BuildingKind, Effect, Job, Unit, JOB_COUNT, UNIT_COUNT};
use crate::game::Game;
use crate::player::PlayerId;
use crate::terrain::Terrain;
use crate::update::RejectReason;

pub const MILLI: i64 = 1000;

/// Répartition voulue de la population, réglée par le joueur.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PopulationPlan {
    /// Part de la population en soldats mobiles (‰).
    pub soldiers_permille: u16,
    /// Part de la population en garnison (‰).
    pub garrison_permille: u16,
    /// Poids relatifs des types de soldats (seuls les types débloqués comptent).
    pub unit_mix: [u16; UNIT_COUNT],
    /// Poids relatifs des métiers ; `None` = répartition automatique.
    pub job_mix: Option<[u16; JOB_COUNT]>,
}

impl Default for PopulationPlan {
    fn default() -> Self {
        PopulationPlan {
            soldiers_permille: 200,
            garrison_permille: 0,
            unit_mix: [1000, 0, 0],
            job_mix: None,
        }
    }
}

impl Cost {
    /// Convertit des unités en millièmes.
    pub fn to_milli(self) -> Cost {
        Cost::new(self.food * MILLI, self.wood * MILLI, self.stone * MILLI, self.iron * MILLI)
    }

    pub fn covers(&self, cost: &Cost) -> bool {
        self.food >= cost.food && self.wood >= cost.wood && self.stone >= cost.stone && self.iron >= cost.iron
    }

    pub fn add(&mut self, other: &Cost) {
        self.food += other.food;
        self.wood += other.wood;
        self.stone += other.stone;
        self.iron += other.iron;
    }

    pub fn pay(&mut self, cost: &Cost) {
        self.food -= cost.food;
        self.wood -= cost.wood;
        self.stone -= cost.stone;
        self.iron -= cost.iron;
    }

    pub fn scaled(&self, num: i64, den: i64) -> Cost {
        Cost::new(
            self.food.saturating_mul(num) / den,
            self.wood.saturating_mul(num) / den,
            self.stone.saturating_mul(num) / den,
            self.iron.saturating_mul(num) / den,
        )
    }

    /// Combien de fois `unit` tient dans ce stock.
    pub fn times(&self, unit: &Cost) -> i64 {
        let mut n = i64::MAX;
        for (have, need) in [
            (self.food, unit.food),
            (self.wood, unit.wood),
            (self.stone, unit.stone),
            (self.iron, unit.iron),
        ] {
            if need > 0 {
                n = n.min(have.max(0) / need);
            }
        }
        n
    }
}

/// Emplois disponibles pour un joueur, par métier.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct JobSlots {
    /// Emplois offerts par le terrain (milli-personnes) et leur rendement.
    pub terrain: [i64; JOB_COUNT],
    /// Emplois offerts par les bâtiments (milli-personnes).
    pub building: [i64; JOB_COUNT],
    /// Production totale si tous les emplois des bâtiments sont pourvus (milli/tour).
    pub building_output: [i64; JOB_COUNT],
}

impl JobSlots {
    pub fn total(&self, job: Job) -> i64 {
        self.terrain[job.index()] + self.building[job.index()]
    }
}

impl Game {
    // ------------------------------------------------------------------
    // Lecture
    // ------------------------------------------------------------------

    /// Logement : population maximale (milli).
    pub fn housing(&self, id: PlayerId) -> i64 {
        let Some(p) = self.player(id) else { return 0 };
        let cfg = &self.config;
        let territory = cfg.housing_factor * icbrt((p.tiles as u64) * (p.tiles as u64)) as i64;
        let buildings: i64 = BuildingKind::ALL
            .iter()
            .map(|&k| p.buildings[k.index()] as i64 * cfg.building(k).housing)
            .sum();
        self.boosted(id, Effect::Housing, (cfg.base_housing + territory + buildings) * MILLI)
    }

    /// Capacité de stockage de chaque ressource (milli).
    pub fn storage(&self, id: PlayerId) -> i64 {
        let cfg = &self.config;
        self.boosted(id, Effect::Storage, (cfg.storage_base * MILLI + self.population(id) * cfg.storage_per_person).max(0))
    }

    /// Soldats partis en campagne ou en mer (milli), par type.
    pub fn away_soldiers(&self, id: PlayerId) -> [i64; UNIT_COUNT] {
        let mut out = [0; UNIT_COUNT];
        for a in self.attacks.iter().filter(|a| a.attacker == id) {
            for u in 0..UNIT_COUNT {
                out[u] += a.units[u];
            }
        }
        for b in self.boats.iter().filter(|b| b.owner == id) {
            for u in 0..UNIT_COUNT {
                out[u] += b.units[u];
            }
        }
        out
    }

    /// Population totale (milli), soldats partis compris.
    pub fn population(&self, id: PlayerId) -> i64 {
        let Some(p) = self.player(id) else { return 0 };
        p.home_population() + self.away_soldiers(id).iter().sum::<i64>()
    }

    pub fn job_slots(&self, id: PlayerId) -> JobSlots {
        let mut slots = JobSlots::default();
        let Some(p) = self.player(id) else { return slots };
        let counts = &self.terrain_tiles[id as usize - 1];
        for job in Job::ALL {
            if !self.job_unlocked(id, job) {
                continue;
            }
            let j = job.index();
            let per_tile = &self.config.jobs_per_tile[j];
            let mut total = 0i64;
            for t in 0..Terrain::COUNT {
                total += counts[t] as i64 * per_tile[t] as i64;
            }
            slots.terrain[j] = total; // ‰ × cases = milli-personnes
        }
        for b in &self.buildings {
            if b.owner != id || b.remaining > 0 {
                continue;
            }
            let def = self.config.building(b.kind);
            if let Some(job) = def.job {
                if self.job_unlocked(id, job) {
                    slots.building[job.index()] += def.jobs * MILLI;
                    slots.building_output[job.index()] += def.jobs * def.job_yield_milli;
                }
            }
        }
        let _ = p;
        slots
    }

    /// Production d'un métier pour un nombre de travailleurs (milli/tour).
    fn job_output(&self, job: Job, workers: i64, slots: &JobSlots) -> i64 {
        let j = job.index();
        let in_buildings = workers.min(slots.building[j]);
        let mut out = 0;
        if in_buildings > 0 {
            out += slots.building_output[j] * in_buildings / slots.building[j];
        }
        let on_land = (workers - in_buildings).clamp(0, slots.terrain[j]);
        out += on_land * self.config.base_yield_milli[j] / MILLI;
        out
    }

    /// Production par tour (milli), avec la répartition actuelle.
    pub fn production(&self, id: PlayerId) -> Cost {
        let Some(p) = self.player(id) else { return Cost::default() };
        let slots = self.job_slots(id);
        let mut out = Cost::default();
        for job in Job::ALL {
            let amount = self.boosted(id, Effect::JobYield(job), self.job_output(job, p.jobs[job.index()], &slots));
            match job.resource() {
                crate::defs::Resource::Food => out.food += amount,
                crate::defs::Resource::Wood => out.wood += amount,
                crate::defs::Resource::Stone => out.stone += amount,
                crate::defs::Resource::Iron => out.iron += amount,
            }
        }
        out
    }

    /// Nourriture consommée par tour (milli).
    pub fn food_consumption(&self, id: PlayerId) -> i64 {
        let Some(p) = self.player(id) else { return 0 };
        let cfg = &self.config;
        let away: i64 = self.away_soldiers(id).iter().sum();
        (p.workers * cfg.worker_food_milli
            + (p.army() + away) * cfg.soldier_food_milli
            + p.garrison * cfg.garrison_food_milli)
            / MILLI
    }

    /// Un métier est débloqué par le premier bâtiment correspondant terminé
    /// (carrière pour les mineurs, mine pour les forgerons).
    pub fn job_unlocked(&self, id: PlayerId, job: Job) -> bool {
        self.player(id).is_some_and(|p| {
            self.config.job_requires[job.index()].is_none_or(|b| p.buildings[b.index()] > 0)
        })
    }

    pub fn unit_unlocked(&self, id: PlayerId, unit: Unit) -> bool {
        let Some(p) = self.player(id) else { return false };
        let def = self.config.unit(unit);
        def.requires.is_none_or(|b| p.buildings[b.index()] > 0)
    }

    /// Coût de formation d'un soldat (milli).
    pub fn soldier_cost(&self, _id: PlayerId, unit: Unit) -> Cost {
        self.config.unit(unit).cost_milli
    }

    /// Personnes formées ou renvoyées par tour au maximum (milli).
    pub fn training_rate(&self, id: PlayerId) -> i64 {
        let Some(p) = self.player(id) else { return 0 };
        let cfg = &self.config;
        let barracks = p.buildings[BuildingKind::Barracks.index()].min(cfg.max_barracks_bonus) as i64;
        let per_100k = cfg.training_per_100k + barracks * cfg.training_per_barracks_per_100k;
        self.boosted(id, Effect::TrainingSpeed, (self.population(id) * per_100k / 100_000).max(MILLI / 2))
    }

    // ------------------------------------------------------------------
    // Ordres
    // ------------------------------------------------------------------

    pub(crate) fn set_plan(&mut self, id: PlayerId, plan: &PopulationPlan) -> Result<(), RejectReason> {
        if plan.soldiers_permille as u32 + plan.garrison_permille as u32 > 1000 {
            return Err(RejectReason::InvalidPlan);
        }
        self.player_mut(id).plan = plan.clone();
        Ok(())
    }

    // ------------------------------------------------------------------
    // Tour
    // ------------------------------------------------------------------

    pub(crate) fn run_population(&mut self) {
        for i in 0..self.players.len() {
            let id = self.players[i].id;
            if !self.players[i].alive || !self.players[i].spawned {
                continue;
            }
            self.produce_and_feed(id);
            self.train(id);
            self.assign_jobs(id);
            self.progress_research(id);
        }
    }

    fn produce_and_feed(&mut self, id: PlayerId) {
        let produced = self.production(id);
        let eaten = self.food_consumption(id);
        let population = self.population(id);
        let housing = self.housing(id);
        let storage = self.storage(id);
        let cfg = &self.config;
        let (growth, margin, famine) = (cfg.growth_per_100k, cfg.auto_food_margin_permille, cfg.famine_per_100k);
        let base_growth = cfg.base_growth_per_100k;

        let p = self.player_mut(id);
        p.resources.add(&produced);
        p.resources.food -= eaten;
        // Au-delà des réserves, la production se perd.
        p.resources.food = p.resources.food.min(storage);
        p.resources.wood = p.resources.wood.min(storage);
        p.resources.stone = p.resources.stone.min(storage);
        p.resources.iron = p.resources.iron.min(storage);

        if p.resources.food < 0 {
            // Famine : on perd des travailleurs, puis des soldats.
            p.resources.food = 0;
            let mut loss = (population * famine / 100_000).max(MILLI / 10);
            let from_workers = loss.min(p.workers);
            p.workers -= from_workers;
            loss -= from_workers;
            if loss > 0 {
                remove_proportional(&mut p.soldiers, loss);
            }
            return;
        }

        // Croissance : pleine vitesse quand l'excédent de nourriture atteint
        // la marge visée, nulle sans excédent. Le stock reste disponible pour
        // former des soldats et construire.
        let surplus = produced.food - eaten;
        if surplus <= 0 || population <= 0 || population >= housing {
            return;
        }
        let full = (eaten * margin / 1000).max(1);
        let factor = (surplus * 1000 / full).min(1000);
        let base = housing * base_growth / 100_000;
        let births = ((population * growth / 100_000 + base) * (housing - population) / housing * factor / 1000)
            .max(MILLI / 20)
            .min(housing - population);
        p.workers += births;
    }

    /// Rapproche soldats et garnison des cibles du plan, au rythme de formation.
    fn train(&mut self, id: PlayerId) {
        let population = self.population(id);
        let rate = self.training_rate(id);
        let away = self.away_soldiers(id);
        let unlocked: Vec<bool> = Unit::ALL.iter().map(|&u| self.unit_unlocked(id, u)).collect();
        let costs: Vec<Cost> = Unit::ALL.iter().map(|&u| self.soldier_cost(id, u)).collect();
        let training_max = self.config.garrison_training_max;
        // Les instructeurs (Académie) accélèrent l'entraînement.
        let training_time = (self.config.garrison_training_time * 1000
            / self.modifier(id, Effect::GarrisonTraining).max(1))
        .max(1);
        let melee_cost = costs[Unit::Melee.index()];

        let p = self.player_mut(id);
        let plan = p.plan.clone();

        // Cibles par type d'unité (seuls les types débloqués).
        let mix_total: i64 = (0..UNIT_COUNT).filter(|&u| unlocked[u]).map(|u| plan.unit_mix[u] as i64).sum();
        let soldiers_target = population * plan.soldiers_permille as i64 / 1000;
        let mut targets = [0i64; UNIT_COUNT];
        for u in 0..UNIT_COUNT {
            if unlocked[u] {
                let share = if mix_total > 0 {
                    plan.unit_mix[u] as i64 * 1000 / mix_total
                } else if u == Unit::Melee.index() {
                    1000
                } else {
                    0
                };
                targets[u] = (soldiers_target * share / 1000 - away[u]).max(0);
            }
        }

        let mut budget = rate;
        // Renvoyer les surplus (gratuit).
        for u in 0..UNIT_COUNT {
            let surplus = p.soldiers[u] - targets[u];
            if surplus > 0 && budget > 0 {
                let n = surplus.min(budget);
                p.soldiers[u] -= n;
                p.workers += n;
                budget -= n;
            }
        }
        // Former les manques (coûte des ressources).
        for _ in 0..UNIT_COUNT {
            let Some(u) = (0..UNIT_COUNT)
                .filter(|&u| targets[u] > p.soldiers[u])
                .max_by_key(|&u| (targets[u] - p.soldiers[u], u))
            else {
                break;
            };
            let deficit = targets[u] - p.soldiers[u];
            let affordable = p.resources.times(&costs[u]).saturating_mul(MILLI);
            let n = deficit.min(budget).min(p.workers).min(affordable);
            if n <= 0 {
                break;
            }
            p.soldiers[u] += n;
            p.workers -= n;
            p.resources.pay(&costs[u].scaled(n, MILLI));
            budget -= n;
        }

        // Garnison.
        let garrison_target = population * plan.garrison_permille as i64 / 1000;
        let garrison_budget = rate;
        if p.garrison > garrison_target {
            let n = (p.garrison - garrison_target).min(garrison_budget);
            p.garrison -= n;
            p.workers += n;
        } else if p.garrison < garrison_target {
            let affordable = p.resources.times(&melee_cost).saturating_mul(MILLI);
            let n = (garrison_target - p.garrison).min(garrison_budget).min(p.workers).min(affordable);
            if n > 0 {
                // Les recrues diluent l'entraînement moyen.
                p.garrison_training = (p.garrison_training * p.garrison + 1000 * n) / (p.garrison + n);
                p.garrison += n;
                p.workers -= n;
                p.resources.pay(&melee_cost.scaled(n, MILLI));
            }
        }
        if p.garrison > 0 {
            if p.garrison_training < training_max {
                let step = ((training_max - p.garrison_training) / training_time).max(1);
                p.garrison_training = (p.garrison_training + step).min(training_max);
            }
        } else {
            p.garrison_training = 1000;
        }
    }

    /// Répartit les travailleurs entre les métiers.
    fn assign_jobs(&mut self, id: PlayerId) {
        let slots = self.job_slots(id);
        let unlocked: Vec<bool> = Job::ALL.iter().map(|&j| self.job_unlocked(id, j)).collect();
        let need_food = self.food_consumption(id) * (1000 + self.config.auto_food_margin_permille) / 1000;
        let base_food_yield = self.config.base_yield_milli[Job::Gatherer.index()];

        let p = self.player_ref(id);
        let workers = p.workers;
        let mut jobs = [0i64; JOB_COUNT];

        match p.plan.job_mix {
            Some(mix) => {
                let total: i64 = (0..JOB_COUNT).filter(|&j| unlocked[j]).map(|j| mix[j] as i64).sum();
                if total == 0 {
                    jobs[Job::Gatherer.index()] = workers;
                } else {
                    let mut given = 0;
                    for j in 0..JOB_COUNT {
                        if unlocked[j] {
                            jobs[j] = workers * mix[j] as i64 / total;
                            given += jobs[j];
                        }
                    }
                    jobs[Job::Gatherer.index()] += workers - given;
                }
            }
            None => {
                // 1. Assez de cueilleurs pour nourrir tout le monde (avec marge).
                let g = Job::Gatherer.index();
                let food_jobs = slots.total(Job::Gatherer);
                let needed = if need_food <= slots.building_output[g] && slots.building_output[g] > 0 {
                    need_food * slots.building[g] / slots.building_output[g]
                } else {
                    slots.building[g]
                        + (need_food - slots.building_output[g]).max(0) * MILLI / base_food_yield.max(1)
                };
                jobs[g] = needed.min(food_jobs).min(workers);
                let mut rest = workers - jobs[g];

                // 2. Le reste par priorités : nourriture (croissance, stock),
                //    bois, pierre, fer, dans la limite des emplois.
                let weights = [(Job::Gatherer, 3), (Job::Lumberjack, 3), (Job::Miner, 4), (Job::Smith, 4)];
                for _ in 0..3 {
                    let open: Vec<(usize, i64, i64)> = weights
                        .iter()
                        .filter(|(j, _)| unlocked[j.index()])
                        .map(|&(j, w)| (j.index(), w, (slots.total(j) - jobs[j.index()]).max(0)))
                        .filter(|&(_, _, free)| free > 0)
                        .collect();
                    let total_w: i64 = open.iter().map(|&(_, w, _)| w).sum();
                    if total_w == 0 || rest <= 0 {
                        break;
                    }
                    let mut given = 0;
                    let round = rest;
                    for (j, w, free) in open {
                        let n = (round * w / total_w).min(free);
                        jobs[j] += n;
                        given += n;
                    }
                    rest -= given;
                    if given == 0 {
                        break;
                    }
                }
                // 3. Les autres cueillent (surplus de nourriture → croissance).
                jobs[g] += rest;
            }
        }
        self.player_mut(id).jobs = jobs;
    }

}

/// Retire `amount` réparti proportionnellement entre les catégories.
pub(crate) fn remove_proportional(values: &mut [i64], amount: i64) {
    let total: i64 = values.iter().sum();
    if total <= 0 || amount <= 0 {
        return;
    }
    if amount >= total {
        values.iter_mut().for_each(|v| *v = 0);
        return;
    }
    let mut removed = 0;
    for v in values.iter_mut() {
        let n = (*v as i128 * amount as i128 / total as i128) as i64;
        *v -= n;
        removed += n;
    }
    // Reste d'arrondi : sur la plus grande catégorie.
    let rest = amount - removed;
    if rest > 0 {
        if let Some(max) = values.iter_mut().max() {
            *max = (*max - rest).max(0);
        }
    }
}

/// Racine cubique entière (par dichotomie).
pub(crate) fn icbrt(value: u64) -> u64 {
    let (mut lo, mut hi) = (0u64, 2_642_245u64); // 2_642_245³ ≈ u64::MAX
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if mid.saturating_mul(mid).saturating_mul(mid) <= value {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    lo
}
