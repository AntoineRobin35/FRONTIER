//! Règles et équilibrage du jeu, sous forme de données.
//!
//! Tous les chiffres d'équilibrage vivent ici. La configuration fait partie
//! de la définition d'une partie : en multijoueur, elle est envoyée à tous
//! les clients avec la carte. Modifier l'équilibrage ne doit jamais demander
//! de toucher au code des systèmes.
//!
//! Conventions d'unités (tout est entier, voir `docs/ARCHITECTURE.md`) :
//! - `‰` (pour mille) : 1000 = ×1 ;
//! - `milli` : millièmes d'unité (1000 = 1 nourriture, 1 soldat de puissance...) ;
//! - les durées sont en tours (10 tours = 1 seconde).

use serde::{Deserialize, Serialize};

use crate::defs::{
    Branch, BuildingKind, Deposit, Effect, Job, TechId, Unit, BUILDING_COUNT, JOB_COUNT, TECH_COUNT, UNIT_COUNT,
};
use crate::terrain::Terrain;

/// Un coût ou un stock de ressources.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Cost {
    pub food: i64,
    pub wood: i64,
    pub stone: i64,
    pub iron: i64,
}

impl Cost {
    pub const fn new(food: i64, wood: i64, stone: i64, iron: i64) -> Cost {
        Cost { food, wood, stone, iron }
    }
}

/// Une technologie de l'Académie.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TechDef {
    pub branch: Branch,
    pub effect: Effect,
    /// Gain par niveau (‰) : +150 = +15 % par niveau.
    pub per_level_permille: i64,
    pub max_level: u8,
    /// Coût du niveau 1 (unités) ; chaque niveau coûte `cost_growth_percent` de plus.
    pub cost: Cost,
    pub cost_growth_percent: u32,
    /// Pierre ajoutée à partir du niveau 2, fer ajouté au niveau 3 (unités).
    pub stone_from_level2: i64,
    pub iron_at_level3: i64,
    pub research_seconds: u32,
    /// Technologie préalable (et son niveau).
    pub requires: Option<(TechId, u8)>,
    /// Bâtiment terminé nécessaire.
    pub requires_building: Option<BuildingKind>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitDef {
    /// Bâtiment nécessaire pour former cette unité.
    pub requires: Option<BuildingKind>,
    /// Coût de formation d'un soldat (milli).
    pub cost_milli: Cost,
    pub attack_permille: i64,
    pub defense_permille: i64,
    /// Efficacité en attaque sur terrain ouvert / accidenté (‰).
    pub open_permille: i64,
    pub rough_permille: i64,
    /// Efficacité contre les cases protégées par un fort (‰).
    pub siege_permille: i64,
    /// Vitesse de progression du front (‰).
    pub speed_permille: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BuildingDef {
    /// Coût du premier exemplaire (unités).
    pub cost: Cost,
    /// Chaque exemplaire possédé renchérit le suivant de ce pourcentage (cumulatif).
    pub cost_growth_percent: u32,
    pub build_ticks: u32,
    /// Gisement requis sous le bâtiment.
    pub deposit: Deposit,
    /// Distance minimale avec un bâtiment du même type.
    pub min_distance: u32,
    /// Nombre maximal par joueur.
    pub max_count: Option<u32>,
    /// Logement ajouté.
    pub housing: i64,
    /// Emplois créés, pour quel métier, avec quel rendement (milli par tour et par travailleur).
    pub job: Option<Job>,
    pub jobs: i64,
    pub job_yield_milli: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GameConfig {
    pub tick_ms: u32,

    // --- Placement -------------------------------------------------------
    pub spawn_radius: u32,
    /// `None` : attendre que tout le monde soit placé (partie solo).
    pub spawn_phase_max_ticks: Option<u32>,
    /// Durée pendant laquelle personne ne peut attaquer un joueur.
    pub immunity_ticks: u32,
    pub start_population: i64,
    /// Part de soldats (guerriers) au départ, et réglage initial du curseur.
    pub start_soldiers_permille: u16,
    pub start_resources: Cost,

    // --- Population ------------------------------------------------------
    /// Logement = base + facteur × cases^(2/3) + bâtiments.
    pub base_housing: i64,
    pub housing_factor: i64,
    /// Naissances par tour = (population × taux + logement × taux de base) / 100 000
    /// × (1 − pop/logement), selon l'excédent de nourriture.
    pub growth_per_100k: i64,
    pub base_growth_per_100k: i64,
    /// Pertes par tour en famine, pour 100 000.
    pub famine_per_100k: i64,
    /// Consommation de nourriture par tour (milli).
    pub worker_food_milli: i64,
    pub soldier_food_milli: i64,
    pub garrison_food_milli: i64,
    /// Formation des soldats : personnes par tour = population × (base + par caserne) / 100 000.
    pub training_per_100k: i64,
    pub training_per_barracks_per_100k: i64,
    pub max_barracks_bonus: u32,
    /// Entraînement de la garnison : de 1000 ‰ à ce maximum, en se rapprochant
    /// de (max − actuel) / constante à chaque tour.
    pub garrison_training_max: i64,
    pub garrison_training_time: i64,
    /// Capacité de stockage de chaque ressource = base + par habitant (unités).
    pub storage_base: i64,
    pub storage_per_person: i64,
    /// Excédent de nourriture visé au-delà des besoins (‰) : c'est aussi
    /// l'excédent qui donne la croissance maximale.
    pub auto_food_margin_permille: i64,

    // --- Économie --------------------------------------------------------
    /// Emplois par case (‰), par métier puis par terrain.
    pub jobs_per_tile: [[u32; Terrain::COUNT]; JOB_COUNT],
    /// Rendement d'un emploi de terrain (milli par tour), par métier.
    pub base_yield_milli: [i64; JOB_COUNT],
    /// Bâtiment terminé qui débloque chaque métier (`None` : dès le départ).
    pub job_requires: [Option<BuildingKind>; JOB_COUNT],

    // --- Définitions -------------------------------------------------------
    pub units: [UnitDef; UNIT_COUNT],
    pub buildings: [BuildingDef; BUILDING_COUNT],
    pub techs: [TechDef; TECH_COUNT],

    // --- Combat ------------------------------------------------------------
    /// Bonus défensif du terrain (‰).
    pub terrain_defense: [i64; Terrain::COUNT],
    /// Terrains « ouverts » (cavalerie à l'aise) ; les autres sont accidentés.
    pub terrain_open: [bool; Terrain::COUNT],
    pub river_defense_permille: i64,
    pub fort_defense_permille: i64,
    pub fort_radius: u32,
    /// Puissance perdue par case prise, avant la densité du défenseur (milli).
    pub base_tile_power_milli: i64,
    /// Poids de la densité du défenseur dans les pertes de l'attaquant (‰).
    /// Comme dans OpenFront : un front dense coûte plus cher, mais l'attaquant
    /// perd moins que la densité qu'il détruit.
    pub density_loss_permille: i64,
    /// Part de sa densité que perd le défenseur à chaque case perdue (‰).
    pub defender_loss_permille: i64,
    /// Bornes du rapport défense / attaque pour les pertes et la vitesse (‰).
    pub loss_ratio_min: i64,
    pub loss_ratio_max: i64,
    pub speed_ratio_min: i64,
    pub speed_ratio_max: i64,
    /// Avancée par tour et par case de front, à forces égales en plaine (‰ de
    /// case). Tout le front avance en même temps, comme dans OpenFront.
    pub attack_speed_permille: i64,
    /// Terres libres : coût par case (milli de puissance, × coût du terrain)…
    pub neutral_tile_power_milli: i64,
    /// … et vitesse par case de front : puissance / échelle, bornée (‰).
    pub neutral_speed_scale: i64,
    pub neutral_speed_min: i64,
    pub neutral_speed_max: i64,
    /// Difficulté de traversée par terrain pour les terres libres (‰, 0 = infranchissable).
    pub terrain_cost: [i64; Terrain::COUNT],
    /// Part des troupes perdue en rappelant une attaque contre un joueur (‰).
    pub retreat_loss_permille: i64,
    pub min_attack_soldiers: i64,
    pub max_tiles_per_tick: u32,

    // --- Bateaux -----------------------------------------------------------
    pub boat_wood: i64,
    pub boat_speed: u32,
    pub max_boats: u32,

    // --- Victoire ----------------------------------------------------------
    pub win_land_permille: u32,
}

impl GameConfig {
    pub fn building(&self, kind: BuildingKind) -> &BuildingDef {
        &self.buildings[kind.index()]
    }

    pub fn unit(&self, unit: Unit) -> &UnitDef {
        &self.units[unit.index()]
    }

    pub fn tech(&self, tech: TechId) -> &TechDef {
        &self.techs[tech.index()]
    }
}

fn per_terrain<T: Copy + Default>(values: &[(Terrain, T)]) -> [T; Terrain::COUNT] {
    let mut out = [T::default(); Terrain::COUNT];
    for &(t, v) in values {
        out[t as usize] = v;
    }
    out
}

fn building(cost: Cost, cost_growth_percent: u32, build_seconds: u32, min_distance: u32) -> BuildingDef {
    BuildingDef {
        cost,
        cost_growth_percent,
        build_ticks: build_seconds * 10,
        deposit: Deposit::None,
        min_distance,
        max_count: None,
        housing: 0,
        job: None,
        jobs: 0,
        job_yield_milli: 0,
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        use Terrain::*;

        let mut buildings = [
            building(Cost::new(0, 600, 0, 0), 100, 30, 18),     // ville
            building(Cost::new(0, 400, 0, 0), 60, 20, 12),      // caserne
            building(Cost::new(0, 250, 0, 0), 25, 15, 6),       // ferme
            building(Cost::new(0, 200, 0, 0), 25, 15, 6),       // scierie
            building(Cost::new(0, 500, 300, 0), 60, 25, 12),    // archerie
            building(Cost::new(500, 600, 300, 0), 60, 25, 12),  // écurie
            building(Cost::new(0, 400, 0, 0), 25, 20, 6),       // carrière
            building(Cost::new(0, 200, 600, 0), 35, 30, 10),    // fort
            building(Cost::new(0, 600, 600, 0), 25, 30, 6),     // mine
            building(Cost::new(400, 800, 0, 0), 0, 40, 0),      // académie
        ];
        let b = &mut buildings;
        b[BuildingKind::City.index()].housing = 2_000;
        b[BuildingKind::Academy.index()].max_count = Some(1);
        let jobs = [
            (BuildingKind::Farm, Deposit::Fertile, Job::Gatherer, 120, 80),
            (BuildingKind::Sawmill, Deposit::Grove, Job::Lumberjack, 120, 20),
            (BuildingKind::Quarry, Deposit::Quarry, Job::Miner, 100, 16),
            (BuildingKind::Mine, Deposit::Iron, Job::Smith, 80, 12),
        ];
        for (kind, deposit, job, count, yield_milli) in jobs {
            let def = &mut b[kind.index()];
            def.deposit = deposit;
            def.job = Some(job);
            def.jobs = count;
            def.job_yield_milli = yield_milli;
        }

        GameConfig {
            tick_ms: 100,

            spawn_radius: 8,
            spawn_phase_max_ticks: None,
            immunity_ticks: 300,
            start_population: 600,
            start_soldiers_permille: 200,
            start_resources: Cost::new(1000, 300, 0, 0),

            base_housing: 1_500,
            housing_factor: 30,
            growth_per_100k: 100,
            base_growth_per_100k: 15,
            famine_per_100k: 200,
            worker_food_milli: 8,
            soldier_food_milli: 12,
            garrison_food_milli: 12,
            training_per_100k: 50,
            training_per_barracks_per_100k: 80,
            max_barracks_bonus: 4,
            garrison_training_max: 3_000,
            garrison_training_time: 4_000,
            auto_food_margin_permille: 300,
            storage_base: 5_000,
            storage_per_person: 3,

            jobs_per_tile: [
                per_terrain(&[
                    (Grass, 2000),
                    (Wetland, 2500),
                    (DryGrass, 1000),
                    (Forest, 1200),
                    // Pêche sur les côtes.
                    (Sand, 1500),
                    (Plateau, 500),
                ]),
                per_terrain(&[
                    (Forest, 2500),
                    (Wetland, 400),
                    (Grass, 500),
                    (DryGrass, 300),
                    (Plateau, 200),
                    (Hills, 150),
                    (Sand, 100),
                ]),
                per_terrain(&[
                    (Hills, 1500),
                    (Plateau, 1200),
                    (Snow, 400),
                    (DryGrass, 200),
                    (Grass, 100),
                    (Forest, 50),
                    (Sand, 200),
                ]),
                per_terrain(&[]),
            ],
            base_yield_milli: [22, 5, 4, 0],
            job_requires: [None, None, Some(BuildingKind::Quarry), Some(BuildingKind::Mine)],

            units: [
                UnitDef {
                    requires: None,
                    cost_milli: Cost::new(500, 500, 0, 0),
                    attack_permille: 1000,
                    defense_permille: 1000,
                    open_permille: 1000,
                    rough_permille: 1000,
                    siege_permille: 1300,
                    speed_permille: 1000,
                },
                UnitDef {
                    requires: Some(BuildingKind::Archery),
                    cost_milli: Cost::new(300, 800, 0, 0),
                    attack_permille: 800,
                    defense_permille: 1400,
                    open_permille: 900,
                    rough_permille: 1250,
                    siege_permille: 1000,
                    speed_permille: 1000,
                },
                UnitDef {
                    requires: Some(BuildingKind::Stable),
                    cost_milli: Cost::new(1500, 500, 0, 0),
                    attack_permille: 1300,
                    defense_permille: 700,
                    open_permille: 1300,
                    rough_permille: 600,
                    siege_permille: 800,
                    speed_permille: 1600,
                },
            ],
            buildings,
            techs: techs(),

            terrain_defense: per_terrain(&[
                (Sand, 900),
                (Grass, 1000),
                (DryGrass, 1000),
                (Forest, 1300),
                (Wetland, 1300),
                (Plateau, 1400),
                (Hills, 1600),
                (Snow, 1500),
            ]),
            terrain_open: per_terrain(&[(Sand, true), (Grass, true), (DryGrass, true)]),
            river_defense_permille: 1300,
            fort_defense_permille: 3000,
            fort_radius: 9,
            base_tile_power_milli: 500,
            density_loss_permille: 400,
            defender_loss_permille: 1000,
            loss_ratio_min: 800,
            loss_ratio_max: 2000,
            speed_ratio_min: 500,
            speed_ratio_max: 4000,
            attack_speed_permille: 120,
            neutral_tile_power_milli: 400,
            neutral_speed_scale: 8_000_000,
            neutral_speed_min: 35,
            neutral_speed_max: 180,
            terrain_cost: per_terrain(&[
                (Sand, 900),
                (Grass, 1000),
                (DryGrass, 1000),
                (Forest, 1400),
                (Wetland, 1300),
                (Plateau, 1500),
                (Hills, 1800),
                (Snow, 2000),
            ]),
            retreat_loss_permille: 250,
            min_attack_soldiers: 10,
            max_tiles_per_tick: 3000,

            boat_wood: 150,
            boat_speed: 3,
            max_boats: 3,

            win_land_permille: 800,
        }
    }
}

/// Table des technologies (voir `defs::TechId` pour l'ordre).
fn techs() -> [TechDef; TECH_COUNT] {
    use Branch::{Economy, Military};
    use BuildingKind as B;
    #[allow(clippy::too_many_arguments)]
    fn t(
        branch: Branch,
        effect: Effect,
        per_level_permille: i64,
        cost: Cost,
        stone_from_level2: i64,
        iron_at_level3: i64,
        research_seconds: u32,
        requires: Option<(TechId, u8)>,
        requires_building: Option<BuildingKind>,
    ) -> TechDef {
        TechDef {
            branch,
            effect,
            per_level_permille,
            max_level: 3,
            cost,
            cost_growth_percent: 80,
            stone_from_level2,
            iron_at_level3,
            research_seconds,
            requires,
            requires_building,
        }
    }
    [
        // Économie
        t(Economy, Effect::JobYield(Job::Gatherer), 150, Cost::new(450, 450, 0, 0), 200, 100, 50, None, None),
        t(Economy, Effect::JobYield(Job::Lumberjack), 150, Cost::new(450, 300, 0, 0), 200, 100, 50, None, None),
        t(Economy, Effect::JobYield(Job::Miner), 150, Cost::new(450, 600, 0, 0), 300, 150, 56, None, Some(B::Quarry)),
        t(Economy, Effect::JobYield(Job::Smith), 150, Cost::new(600, 750, 0, 0), 400, 200, 62, None, Some(B::Mine)),
        t(Economy, Effect::BuildSpeed, 250, Cost::new(300, 750, 0, 0), 200, 100, 56, None, None),
        t(Economy, Effect::Housing, 100, Cost::new(900, 900, 0, 0), 400, 200, 75, Some((TechId::Architecture, 1)), None),
        t(Economy, Effect::Storage, 300, Cost::new(450, 450, 0, 0), 150, 50, 37, None, None),
        // Militaire
        t(Military, Effect::TrainingSpeed, 250, Cost::new(600, 450, 0, 0), 200, 100, 56, None, None),
        t(Military, Effect::UnitAttack(Unit::Melee), 150, Cost::new(600, 600, 0, 0), 300, 200, 62, None, None),
        t(Military, Effect::UnitAttack(Unit::Ranged), 150, Cost::new(450, 750, 0, 0), 300, 200, 62, None, Some(B::Archery)),
        t(Military, Effect::UnitAttack(Unit::Cavalry), 150, Cost::new(900, 450, 0, 0), 300, 200, 62, None, Some(B::Stable)),
        t(Military, Effect::SoldierDefense, 120, Cost::new(600, 750, 0, 0), 300, 200, 62, None, None),
        t(Military, Effect::GarrisonDefense, 150, Cost::new(750, 450, 0, 0), 300, 200, 62, None, None),
        t(Military, Effect::GarrisonTraining, 400, Cost::new(600, 450, 0, 0), 200, 100, 56, Some((TechId::Discipline, 1)), None),
        t(Military, Effect::NeutralSpeed, 250, Cost::new(450, 450, 0, 0), 150, 50, 37, None, None),
        t(Military, Effect::RetreatLoss, -250, Cost::new(450, 600, 0, 0), 200, 100, 50, None, None),
        t(Military, Effect::FortDefense, 200, Cost::new(450, 450, 450, 0), 400, 200, 62, None, Some(B::Fort)),
    ]
}
