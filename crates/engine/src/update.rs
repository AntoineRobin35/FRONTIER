use serde::{Deserialize, Serialize};

use crate::defs::{BuildingKind, TechId};
use crate::map::TileIdx;
use crate::player::PlayerId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectReason {
    WrongPhase,
    UnknownPlayer,
    NotAlive,
    AlreadySpawned,
    InvalidTile,
    TileTaken,
    InvalidTarget,
    NotEnoughTroops,
    NoBorder,
    UnknownAttack,
    NotOwned,
    TooClose,
    NotEnoughResources,
    TooManyBoats,
    NoPath,
    Immune,
    NeedsDeposit,
    Researching,
    InvalidPlan,
    MaxCount,
    NoAcademy,
    MaxLevel,
    Locked,
}

/// Ce qui s'est passé pendant un tour : sert au journal, à l'interface et
/// aux IA. Ne contient jamais d'information nécessaire pour reconstruire
/// l'état (l'état est dans `Game`). Les effectifs sont en personnes entières.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GameEvent {
    Spawned { player: PlayerId, tile: TileIdx },
    GameStarted,
    AttackStarted { attack_id: u32, attacker: PlayerId, target: PlayerId, troops: i64 },
    AttackReinforced { attack_id: u32, attacker: PlayerId, target: PlayerId, troops: i64 },
    AttacksClashed { a: PlayerId, b: PlayerId, troops: i64 },
    AttackEnded { attack_id: u32, attacker: PlayerId, target: PlayerId, returned: i64 },
    BoatLaunched { boat_id: u32, owner: PlayerId, tile: TileIdx, troops: i64 },
    BoatLanded { boat_id: u32, owner: PlayerId, tile: TileIdx },
    BoatLost { boat_id: u32, owner: PlayerId },
    BuildStarted { building_id: u32, owner: PlayerId, kind: BuildingKind, tile: TileIdx },
    BuildingCompleted { building_id: u32, owner: PlayerId, kind: BuildingKind, tile: TileIdx },
    BuildingCaptured { building_id: u32, kind: BuildingKind, from: PlayerId, to: PlayerId },
    BuildingDestroyed { building_id: u32, kind: BuildingKind, owner: PlayerId },
    ResearchStarted { player: PlayerId, tech: TechId, level: u8 },
    ResearchCompleted { player: PlayerId, tech: TechId, level: u8 },
    Rejected { player: PlayerId, reason: RejectReason },
    Eliminated { player: PlayerId, by: Option<PlayerId> },
    Victory { winner: Option<PlayerId> },
}

/// Résultat d'un tour de simulation.
#[derive(Clone, Debug, Default)]
pub struct TickUpdate {
    /// Numéro du tour qui vient d'être calculé.
    pub tick: u64,
    /// Cases dont le propriétaire a changé (une case peut apparaître deux fois).
    pub changed_tiles: Vec<TileIdx>,
    pub events: Vec<GameEvent>,
}
