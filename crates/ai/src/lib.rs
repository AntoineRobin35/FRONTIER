//! IA de FRONTIER.
//!
//! Une IA est un **joueur comme un autre** : elle lit l'état de la partie
//! (le même que celui affiché à l'écran) et renvoie des ordres (`Intent`).
//! Elle ne peut ni tricher ni modifier la partie directement.
//!
//! Pour créer une nouvelle IA :
//! 1. créer un fichier dans `src/agents/` avec une structure qui implémente
//!    le trait [`Agent`] ;
//! 2. l'ajouter dans [`AGENTS`] et dans [`create_agent`].
//!
//! Elle apparaît alors automatiquement dans le menu du jeu.
//! Plus tard, des IA externes (Python, réseau de neurones...) pourront se
//! brancher via la même interface.

mod agents;
mod core;
pub mod sense;
pub mod spawn;

use frontier_engine::{Game, Intent, PlayerId};
use serde::{Deserialize, Serialize};

pub use crate::core::{BotCore, NavalGoal};

/// Contrat de toute IA.
pub trait Agent: Send {
    /// Appelée à chaque tour, avant la simulation. L'IA ajoute ses ordres
    /// dans `out` (souvent aucun : réfléchir à chaque tour n'est pas utile).
    fn act(&mut self, game: &Game, me: PlayerId, out: &mut Vec<Intent>);
}

/// Niveau de difficulté. Les niveaux plus élevés sont volontairement
/// laissés aux IA avancées à venir.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Easy,
    Normal,
}

/// Description d'une IA disponible, affichée dans le menu.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct AgentInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

pub const AGENTS: &[AgentInfo] = &[
    AgentInfo {
        id: "colon",
        name: "Colon",
        description: "Bâtisseur : économie, gisements et villes d'abord ; se défend, attaque rarement.",
    },
    AgentInfo {
        id: "conquerant",
        name: "Conquérant",
        description: "Guerrier : grosse armée et cavalerie, attaque le voisin le moins défendu.",
    },
    AgentInfo {
        id: "opportuniste",
        name: "Opportuniste",
        description: "Équilibré : économie et armée mixte, riposte et profite des faiblesses.",
    },
    AgentInfo {
        id: "bastion",
        name: "Bastion",
        description: "Tortue : garnison entraînée et forts, puis frappe une fois aguerri.",
    },
    AgentInfo {
        id: "passif",
        name: "Passif",
        description: "Se place et ne fait rien. Pour s'entraîner ou tester.",
    },
];

/// Crée une IA à partir de son identifiant. `seed` rend son comportement
/// reproductible.
pub fn create_agent(id: &str, difficulty: Difficulty, seed: u64) -> Option<Box<dyn Agent>> {
    let core = BotCore::new(difficulty, seed);
    let agent: Box<dyn Agent> = match id {
        "colon" => Box::new(agents::Colon::new(core)),
        "conquerant" => Box::new(agents::Conquerant::new(core)),
        "opportuniste" => Box::new(agents::Opportuniste::new(core)),
        "bastion" => Box::new(agents::Bastion::new(core)),
        "passif" => Box::new(agents::Passif::new(core)),
        _ => return None,
    };
    Some(agent)
}

/// Fait jouer un ensemble d'IA.
#[derive(Default)]
pub struct BotHost {
    bots: Vec<(PlayerId, Box<dyn Agent>)>,
    buffer: Vec<Intent>,
}

impl BotHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, player: PlayerId, agent: Box<dyn Agent>) {
        self.bots.push((player, agent));
    }

    pub fn len(&self) -> usize {
        self.bots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bots.is_empty()
    }

    /// Demande à chaque IA vivante ses ordres pour le prochain tour.
    pub fn collect(&mut self, game: &Game, out: &mut Vec<(PlayerId, Intent)>) {
        for (player, agent) in &mut self.bots {
            if !game.player(*player).is_some_and(|p| p.alive) {
                continue;
            }
            self.buffer.clear();
            agent.act(game, *player, &mut self.buffer);
            out.extend(self.buffer.drain(..).map(|intent| (*player, intent)));
        }
    }
}
