use serde::{Deserialize, Serialize};

use crate::defs::{BuildingKind, TechId};
use crate::map::TileIdx;
use crate::player::PlayerId;
use crate::population::PopulationPlan;

/// Un ordre donné par un joueur, humain ou IA.
///
/// C'est **la seule façon d'agir sur la partie**. En multijoueur, seuls les
/// ordres circulent sur le réseau ; une IA ne peut rien faire de plus qu'un
/// humain. On peut ajouter des variantes, mais une variante existante ne
/// doit pas changer de sens.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Intent {
    /// Choisir son point de départ (phase de placement uniquement).
    Spawn { tile: TileIdx },
    /// Envoyer une part de ses soldats contre un voisin (`0` = terres neutres).
    Attack { target: PlayerId, ratio_permille: u16 },
    /// Rappeler une attaque (coûteux face à un joueur).
    CancelAttack { attack_id: u32 },
    /// Envoyer une part de ses soldats par bateau vers une case côtière.
    Naval { tile: TileIdx, ratio_permille: u16 },
    /// Lancer un chantier sur une de ses cases.
    Build { tile: TileIdx, kind: BuildingKind },
    /// Changer la répartition de sa population.
    SetPlan { plan: PopulationPlan },
    /// Lancer une recherche à l'Académie (niveau suivant).
    Research { tech: TechId },
}
