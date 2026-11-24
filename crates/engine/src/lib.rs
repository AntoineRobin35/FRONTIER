//! FRONTIER — moteur de simulation.
//!
//! Le moteur est **pur et déterministe** : à partir d'une carte, d'une
//! configuration et d'une suite d'ordres (`Intent`), il produit toujours
//! exactement le même état, sur n'importe quelle machine. C'est la base du
//! multijoueur (chaque client rejoue les mêmes ordres) et de l'entraînement
//! d'IA (simulation sans affichage).
//!
//! Règles à respecter dans ce crate (voir `docs/ARCHITECTURE.md`) :
//! - pas de flottants dans la simulation, uniquement des entiers ;
//! - pas d'itération sur `HashMap`/`HashSet` (ordre non déterministe) ;
//! - pas d'horloge, pas d'aléatoire système : uniquement `rng::Rng` ;
//! - aucune entrée/sortie : le moteur ne connaît ni l'écran ni le réseau.
//!
//! Organisation : `game` contient l'état et l'ordre d'un tour ; chaque
//! système (`spawn`, `population`, `research`, `attack`, `naval`, `buildings`) ajoute ses
//! méthodes à `Game` dans son propre fichier.

pub mod config;
pub mod defs;
pub mod game;
pub mod intent;
pub mod map;
pub mod player;
pub mod rng;
pub mod terrain;
pub mod update;

mod attack;
mod buildings;
mod insight;
mod naval;
mod population;
mod research;
mod spawn;

#[cfg(test)]
mod tests;

pub use attack::Attack;
pub use buildings::Building;
pub use insight::{AttackEstimate, PlanOutcome};
pub use config::{BuildingDef, Cost, GameConfig, TechDef, UnitDef};
pub use defs::{Branch, BuildingKind, Deposit, Effect, Job, Resource, TechId, Unit};
pub use population::{JobSlots, PopulationPlan, MILLI};
pub use game::{Game, GameSetup, Phase, PlayerSetup, SetupError, MAX_PLAYERS};
pub use intent::Intent;
pub use map::{DepositSite, MapData, MapError, Regions, RiverPath, TileIdx, NO_REGION};
pub use naval::Boat;
pub use player::{Player, PlayerId, Research, NEUTRAL};
pub use terrain::Terrain;
pub use update::{GameEvent, RejectReason, TickUpdate};
