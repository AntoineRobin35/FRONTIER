//! Les « catégories » du jeu : ressources, métiers, unités, gisements,
//! bâtiments, technologies. Leurs valeurs numériques sont stables (utilisées
//! par le rendu, le réseau et les IA) : on ajoute à la fin, on ne réordonne
//! jamais. Les chiffres (coûts, rendements...) sont dans `GameConfig`.

use serde::{Deserialize, Serialize};

macro_rules! all_variants {
    ($ty:ident, $count:ident, [$($v:ident),* $(,)?]) => {
        impl $ty {
            pub const ALL: [$ty; $count] = [$($ty::$v),*];
            #[inline]
            pub fn index(self) -> usize {
                self as usize
            }
        }
    };
}

pub const RESOURCE_COUNT: usize = 4;
pub const JOB_COUNT: usize = 4;
pub const UNIT_COUNT: usize = 3;
pub const DEPOSIT_COUNT: usize = 5;
pub const BUILDING_COUNT: usize = 10;
pub const TECH_COUNT: usize = 17;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resource {
    Food = 0,
    Wood = 1,
    Stone = 2,
    Iron = 3,
}
all_variants!(Resource, RESOURCE_COUNT, [Food, Wood, Stone, Iron]);

/// Métiers des travailleurs.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Job {
    /// Cueilleur-chasseur : nourriture.
    Gatherer = 0,
    /// Bûcheron : bois.
    Lumberjack = 1,
    /// Mineur : pierre (débloqué par la première carrière).
    Miner = 2,
    /// Forgeron : fer (débloqué par la première mine).
    Smith = 3,
}
all_variants!(Job, JOB_COUNT, [Gatherer, Lumberjack, Miner, Smith]);

impl Job {
    pub fn resource(self) -> Resource {
        match self {
            Job::Gatherer => Resource::Food,
            Job::Lumberjack => Resource::Wood,
            Job::Miner => Resource::Stone,
            Job::Smith => Resource::Iron,
        }
    }
}

/// Types de soldats mobiles (la garnison est à part : elle ne bouge pas).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    /// Guerrier, corps à corps.
    Melee = 0,
    /// Archer, à distance.
    Ranged = 1,
    /// Cavalier.
    Cavalry = 2,
}
all_variants!(Unit, UNIT_COUNT, [Melee, Ranged, Cavalry]);

/// Gisement d'une case.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Deposit {
    #[default]
    None = 0,
    /// Terres fertiles : ferme.
    Fertile = 1,
    /// Bosquet : scierie.
    Grove = 2,
    /// Pierre : carrière.
    Quarry = 3,
    /// Fer : mine.
    Iron = 4,
}
all_variants!(Deposit, DEPOSIT_COUNT, [None, Fertile, Grove, Quarry, Iron]);

impl Deposit {
    pub fn from_u8(v: u8) -> Deposit {
        Self::ALL.get(v as usize).copied().unwrap_or(Deposit::None)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingKind {
    /// Logement : augmente la population maximale.
    City = 0,
    /// Forme les soldats plus vite.
    Barracks = 1,
    /// Sur terres fertiles : emplois de cueilleurs très productifs.
    Farm = 2,
    /// Sur bosquet : emplois de bûcherons très productifs.
    Sawmill = 3,
    /// Débloque les archers.
    Archery = 4,
    /// Débloque les cavaliers.
    Stable = 5,
    /// Sur gisement de pierre : débloque les mineurs.
    Quarry = 6,
    /// Défense ×3 dans un rayon.
    Fort = 7,
    /// Sur gisement de fer : débloque les forgerons.
    Mine = 8,
    /// Recherches (un seul exemplaire).
    Academy = 9,
}
all_variants!(
    BuildingKind,
    BUILDING_COUNT,
    [City, Barracks, Farm, Sawmill, Archery, Stable, Quarry, Fort, Mine, Academy]
);

/// Branche de l'Académie.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Branch {
    Economy,
    Military,
}

/// Technologies de l'Académie.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechId {
    Agriculture = 0,
    Forestry = 1,
    Masonry = 2,
    Metallurgy = 3,
    Architecture = 4,
    Urbanism = 5,
    Granaries = 6,
    Drill = 7,
    Blades = 8,
    Bowmanship = 9,
    Horsemanship = 10,
    Shields = 11,
    Discipline = 12,
    Instructors = 13,
    Colonization = 14,
    Logistics = 15,
    Fortification = 16,
}
all_variants!(
    TechId,
    TECH_COUNT,
    [
        Agriculture,
        Forestry,
        Masonry,
        Metallurgy,
        Architecture,
        Urbanism,
        Granaries,
        Drill,
        Blades,
        Bowmanship,
        Horsemanship,
        Shields,
        Discipline,
        Instructors,
        Colonization,
        Logistics,
        Fortification
    ]
);

/// Ce qu'une technologie améliore. Chaque effet est un multiplicateur (‰)
/// lu par le système concerné via `Game::modifier`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "of", rename_all = "snake_case")]
pub enum Effect {
    /// Rendement d'un métier.
    JobYield(Job),
    /// Vitesse des chantiers.
    BuildSpeed,
    /// Logement.
    Housing,
    /// Capacité des réserves.
    Storage,
    /// Vitesse de formation des soldats.
    TrainingSpeed,
    /// Attaque d'un type d'unité.
    UnitAttack(Unit),
    /// Défense de tous les soldats mobiles.
    SoldierDefense,
    /// Défense de la garnison.
    GarrisonDefense,
    /// Vitesse d'entraînement de la garnison.
    GarrisonTraining,
    /// Vitesse d'expansion sur les terres libres.
    NeutralSpeed,
    /// Pertes en cas de retraite (valeur négative = moins de pertes).
    RetreatLoss,
    /// Défense apportée par les forts.
    FortDefense,
}
