use frontier_engine::{BuildingKind::*, Game, Intent, PlayerId, PopulationPlan, TechId};

use super::{expand, retaliate, strike_weakest};
use crate::sense::sense;
use crate::{Agent, BotCore};

/// Tortue : forte garnison qui s'entraîne, forts sur les frontières. Une
/// fois la garnison aguerrie et l'âge de Pierre atteint, il lève une armée
/// et frappe sans craindre la riposte. Vulnérable s'il se développe mal.
pub struct Bastion {
    core: BotCore,
}

impl Bastion {
    pub fn new(core: BotCore) -> Self {
        Bastion { core }
    }
}

const BUILD_ORDER: &[(frontier_engine::BuildingKind, u32)] = &[
    (Farm, 1),
    (Sawmill, 1),
    (Academy, 1),
    (City, 1),
    (Quarry, 1),
    (Barracks, 1),
    (Farm, 2),
    (City, 2),
    (Archery, 1),
    (Quarry, 2),
    (Mine, 1),
];

/// La garnison d'abord, puis les forts.
const RESEARCH: &[TechId] = &[
    TechId::Discipline,
    TechId::Instructors,
    TechId::Agriculture,
    TechId::Discipline,
    TechId::Instructors,
    TechId::Fortification,
    TechId::Shields,
    TechId::Bowmanship,
    TechId::Discipline,
    TechId::Masonry,
    TechId::Fortification,
];

impl Agent for Bastion {
    fn act(&mut self, game: &Game, me: PlayerId, out: &mut Vec<Intent>) {
        if self.core.handle_spawn(game, me, out) || !self.core.should_think(game) {
            return;
        }
        let s = sense(game, me);
        let Some(p) = game.player(me) else { return };
        let ready = p.buildings[Quarry.index()] > 0 && p.garrison_training >= 2_200;
        let plan = PopulationPlan {
            soldiers_permille: if ready { 300 } else { 100 },
            garrison_permille: 250,
            unit_mix: BotCore::unit_mix(game, me, [400, 600, 0]),
            job_mix: None,
        };
        self.core.ensure_plan(game, me, plan, out);

        retaliate(&self.core, game, me, &s, out);
        // Un fort face à chaque voisin, dès que possible.
        for enemy in s.neighbors().collect::<Vec<_>>() {
            if game.owned_count(me, Fort) < 4 && self.core.try_build_fort(game, me, enemy, &s, out) {
                return;
            }
        }
        if self.core.try_research(game, me, RESEARCH, out) || self.core.follow_build_order(game, me, BUILD_ORDER, &s, out) {
            return;
        }
        if expand(&self.core, game, me, &s, 500, out) {
            return;
        }
        if ready {
            strike_weakest(&self.core, game, me, &s, 800, 1000, out);
        }
    }
}
