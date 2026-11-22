use serde::{Deserialize, Serialize};

use crate::config::Cost;
use crate::defs::{TechId, BUILDING_COUNT, JOB_COUNT, TECH_COUNT, UNIT_COUNT};
use crate::map::TileIdx;
use crate::population::PopulationPlan;

/// Identifiant d'un joueur. `0` est réservé aux terres neutres, les joueurs
/// sont numérotés à partir de 1.
pub type PlayerId = u16;

pub const NEUTRAL: PlayerId = 0;

/// État d'un joueur. Le moteur ne sait pas si un joueur est un humain ou une
/// IA : c'est le rôle de la session.
///
/// Les personnes et les ressources sont comptées en **millièmes** (entiers) :
/// `1000` = une personne, une unité de bois...
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub tiles: u32,
    pub spawned: bool,
    pub alive: bool,
    pub spawn_tile: Option<TileIdx>,

    /// Stocks (milli).
    pub resources: Cost,
    /// Niveau atteint dans chaque technologie de l'Académie.
    pub techs: [u8; TECH_COUNT],
    /// Recherche en cours.
    pub research: Option<Research>,

    /// Travailleurs (milli), et leur répartition par métier ce tour-ci.
    pub workers: i64,
    pub jobs: [i64; JOB_COUNT],
    /// Soldats mobiles à la maison (milli), par type.
    pub soldiers: [i64; UNIT_COUNT],
    /// Garnison (milli) et son niveau d'entraînement (‰, de 1000 à 3000).
    pub garrison: i64,
    pub garrison_training: i64,
    pub plan: PopulationPlan,

    /// Bâtiments terminés, par type.
    pub buildings: [u32; BUILDING_COUNT],
}

/// Une recherche en cours à l'Académie.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Research {
    pub tech: TechId,
    /// Niveau visé.
    pub level: u8,
    pub remaining: u32,
    pub total: u32,
}

impl Player {
    pub(crate) fn new(id: PlayerId, name: String) -> Self {
        Player {
            id,
            name,
            tiles: 0,
            spawned: false,
            alive: true,
            spawn_tile: None,
            resources: Cost::default(),
            techs: [0; TECH_COUNT],
            research: None,
            workers: 0,
            jobs: [0; JOB_COUNT],
            soldiers: [0; UNIT_COUNT],
            garrison: 0,
            garrison_training: 1000,
            plan: PopulationPlan::default(),
            buildings: [0; BUILDING_COUNT],
        }
    }

    /// Soldats mobiles à la maison (milli).
    pub fn army(&self) -> i64 {
        self.soldiers.iter().sum()
    }

    /// Population présente sur le territoire (milli), hors soldats partis.
    pub fn home_population(&self) -> i64 {
        self.workers + self.army() + self.garrison
    }
}
