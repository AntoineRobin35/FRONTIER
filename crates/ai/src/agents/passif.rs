use frontier_engine::{Game, Intent, PlayerId};

use crate::{Agent, BotCore};

/// L'IA la plus simple possible : se placer, puis ne rien faire.
/// Bon point de départ pour écrire sa propre IA.
pub struct Passif {
    core: BotCore,
}

impl Passif {
    pub fn new(core: BotCore) -> Self {
        Passif { core }
    }
}

impl Agent for Passif {
    fn act(&mut self, game: &Game, me: PlayerId, out: &mut Vec<Intent>) {
        self.core.handle_spawn(game, me, out);
    }
}
