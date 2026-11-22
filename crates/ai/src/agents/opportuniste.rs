use frontier_engine::{BuildingKind::*, Game, Intent, PlayerId, PopulationPlan, TechId};

use super::{expand, retaliate, strike_weakest};
use crate::sense::sense;
use crate::{Agent, BotCore, NavalGoal};

/// Équilibré : économie et armée mixte, riposte, et profite d'un voisin
/// nettement moins défendu, à pied ou par la mer.
pub struct Opportuniste {
    core: BotCore,
}

impl Opportuniste {
    pub fn new(core: BotCore) -> Self {
        Opportuniste { core }
    }
}

const BUILD_ORDER: &[(frontier_engine::BuildingKind, u32)] = &[
    (Sawmill, 1),
    (Farm, 1),
    (City, 1),
    (Academy, 1),
    (Barracks, 1),
    (Quarry, 1),
    (Farm, 2),
    (Archery, 1),
    (City, 2),
    (Sawmill, 2),
    (Stable, 1),
    (Mine, 1),
    (City, 3),
];

/// Un peu de tout.
const RESEARCH: &[TechId] = &[
    TechId::Agriculture,
    TechId::Blades,
    TechId::Colonization,
    TechId::Architecture,
    TechId::Bowmanship,
    TechId::Shields,
    TechId::Drill,
    TechId::Forestry,
    TechId::Horsemanship,
    TechId::Masonry,
    TechId::Blades,
    TechId::Urbanism,
];

impl Agent for Opportuniste {
    fn act(&mut self, game: &Game, me: PlayerId, out: &mut Vec<Intent>) {
        if self.core.handle_spawn(game, me, out) || !self.core.should_think(game) {
            return;
        }
        let s = sense(game, me);
        let plan = PopulationPlan {
            soldiers_permille: 250,
            garrison_permille: 0,
            unit_mix: BotCore::unit_mix(game, me, [500, 300, 200]),
            job_mix: None,
        };
        self.core.ensure_plan(game, me, plan, out);

        if let Some(enemy) = retaliate(&self.core, game, me, &s, out) {
            if game.owned_count(me, Fort) < 5 {
                self.core.try_build_fort(game, me, enemy, &s, out);
            }
        }
        if strike_weakest(&self.core, game, me, &s, 600, 1100, out) {
            return;
        }
        if self.core.try_research(game, me, RESEARCH, out) || self.core.follow_build_order(game, me, BUILD_ORDER, &s, out) {
            return;
        }
        if expand(&self.core, game, me, &s, 500, out) {
            return;
        }
        if s.neutral_border() == 0 && s.boats_at_sea == 0 {
            if let Some(tile) = self.core.naval_target(game, me, &s, NavalGoal::Any) {
                out.push(Intent::Naval { tile, ratio_permille: self.core.ratio(450) });
            }
        }
    }
}
