use frontier_engine::{BuildingKind::*, Game, Intent, PlayerId, PopulationPlan, TechId};

use super::{expand, retaliate, strike_weakest};
use crate::sense::sense;
use crate::{Agent, BotCore, NavalGoal};

/// Bâtisseur pacifique : économie d'abord, gisements, villes, colonies par
/// bateau. Se défend (forts, archers) et n'attaque que s'il est écrasant.
pub struct Colon {
    core: BotCore,
}

impl Colon {
    pub fn new(core: BotCore) -> Self {
        Colon { core }
    }
}

const BUILD_ORDER: &[(frontier_engine::BuildingKind, u32)] = &[
    (Sawmill, 1),
    (Farm, 1),
    (City, 1),
    (Academy, 1),
    (Sawmill, 2),
    (Farm, 2),
    (Quarry, 1),
    (Barracks, 1),
    (City, 2),
    (Farm, 3),
    (Archery, 1),
    (Quarry, 2),
    (Mine, 1),
    (City, 3),
    (Sawmill, 3),
];

/// Économie d'abord, puis défense.
const RESEARCH: &[TechId] = &[
    TechId::Agriculture,
    TechId::Forestry,
    TechId::Architecture,
    TechId::Urbanism,
    TechId::Granaries,
    TechId::Masonry,
    TechId::Agriculture,
    TechId::Shields,
    TechId::Bowmanship,
    TechId::Colonization,
    TechId::Urbanism,
    TechId::Forestry,
    TechId::Metallurgy,
    TechId::Fortification,
];

impl Agent for Colon {
    fn act(&mut self, game: &Game, me: PlayerId, out: &mut Vec<Intent>) {
        if self.core.handle_spawn(game, me, out) || !self.core.should_think(game) {
            return;
        }
        let s = sense(game, me);
        let threatened = !s.incoming.is_empty();
        let plan = PopulationPlan {
            soldiers_permille: if threatened { 300 } else { 170 },
            garrison_permille: if game.owned_count(me, Academy) > 0 { 60 } else { 0 },
            unit_mix: BotCore::unit_mix(game, me, [600, 400, 0]),
            job_mix: None,
        };
        self.core.ensure_plan(game, me, plan, out);

        if let Some(enemy) = retaliate(&self.core, game, me, &s, out) {
            if game.owned_count(me, Fort) < 6 && self.core.try_build_fort(game, me, enemy, &s, out) {
                return;
            }
        }
        if self.core.try_research(game, me, RESEARCH, out) || self.core.follow_build_order(game, me, BUILD_ORDER, &s, out) {
            return;
        }
        if expand(&self.core, game, me, &s, 500, out) {
            return;
        }
        if s.neutral_border() == 0 && s.boats_at_sea == 0 {
            if let Some(tile) = self.core.naval_target(game, me, &s, NavalGoal::Settle) {
                out.push(Intent::Naval { tile, ratio_permille: self.core.ratio(400) });
                return;
            }
        }
        strike_weakest(&self.core, game, me, &s, 600, 2000, out);
    }
}
