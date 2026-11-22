use frontier_engine::{BuildingKind::*, Game, Intent, PlayerId, PopulationPlan, TechId};

use super::{expand, retaliate, strike_weakest};
use crate::sense::sense;
use crate::{Agent, BotCore, NavalGoal};

/// Guerrier : grosse armée, casernes et écuries, s'étend vite et attaque le
/// voisin le moins défendu dès qu'il le surpasse. Économie plus faible.
pub struct Conquerant {
    core: BotCore,
}

impl Conquerant {
    pub fn new(core: BotCore) -> Self {
        Conquerant { core }
    }
}

const BUILD_ORDER: &[(frontier_engine::BuildingKind, u32)] = &[
    (Barracks, 1),
    (Sawmill, 1),
    (Farm, 1),
    (City, 1),
    (Academy, 1),
    (Barracks, 2),
    (Quarry, 1),
    (Stable, 1),
    (City, 2),
    (Farm, 2),
    (Mine, 1),
    (Barracks, 3),
];

/// L'armée d'abord.
const RESEARCH: &[TechId] = &[
    TechId::Drill,
    TechId::Blades,
    TechId::Colonization,
    TechId::Horsemanship,
    TechId::Blades,
    TechId::Agriculture,
    TechId::Drill,
    TechId::Horsemanship,
    TechId::Logistics,
    TechId::Shields,
    TechId::Blades,
    TechId::Horsemanship,
];

impl Agent for Conquerant {
    fn act(&mut self, game: &Game, me: PlayerId, out: &mut Vec<Intent>) {
        if self.core.handle_spawn(game, me, out) || !self.core.should_think(game) {
            return;
        }
        let s = sense(game, me);
        let plan = PopulationPlan {
            soldiers_permille: 350,
            garrison_permille: 0,
            unit_mix: BotCore::unit_mix(game, me, [500, 0, 500]),
            job_mix: None,
        };
        self.core.ensure_plan(game, me, plan, out);

        retaliate(&self.core, game, me, &s, out);
        if strike_weakest(&self.core, game, me, &s, 700, 800, out) {
            return;
        }
        if expand(&self.core, game, me, &s, 600, out) {
            return;
        }
        if self.core.try_research(game, me, RESEARCH, out) || self.core.follow_build_order(game, me, BUILD_ORDER, &s, out) {
            return;
        }
        if s.neutral_border() == 0 && s.boats_at_sea == 0 && s.neighbors().next().is_none() {
            if let Some(tile) = self.core.naval_target(game, me, &s, NavalGoal::Any) {
                out.push(Intent::Naval { tile, ratio_permille: self.core.ratio(500) });
            }
        }
    }
}
