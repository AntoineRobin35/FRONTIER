//! Académie : recherches et bonus.
//!
//! Une seule recherche à la fois, payée avec les ressources ordinaires et
//! qui prend du temps. Chaque technologie a des niveaux ; chaque niveau
//! augmente un effet (`Effect`). Tous les systèmes lisent ces bonus via
//! [`Game::modifier`] : c'est le seul point d'entrée, ce qui garde le code
//! simple et permet d'ajouter des technologies sans toucher aux systèmes.

use crate::config::Cost;
use crate::defs::{BuildingKind, Effect, TechId};
use crate::game::{Game, Phase};
use crate::player::{PlayerId, Research};
use crate::update::{GameEvent, RejectReason};

impl Game {
    /// Multiplicateur d'un effet pour un joueur (‰ : 1000 = aucun bonus).
    pub fn modifier(&self, id: PlayerId, effect: Effect) -> i64 {
        let Some(p) = self.player(id) else { return 1000 };
        let mut m = 1000;
        for tech in TechId::ALL {
            let level = p.techs[tech.index()] as i64;
            if level == 0 {
                continue;
            }
            let def = self.config.tech(tech);
            if def.effect == effect {
                m += level * def.per_level_permille;
            }
        }
        m.max(0)
    }

    /// Applique un effet à une valeur.
    #[inline]
    pub(crate) fn boosted(&self, id: PlayerId, effect: Effect, value: i64) -> i64 {
        value * self.modifier(id, effect) / 1000
    }

    pub fn tech_level(&self, id: PlayerId, tech: TechId) -> u8 {
        self.player(id).map_or(0, |p| p.techs[tech.index()])
    }

    /// Coût (milli) et durée (tours) du prochain niveau, ou `None` au maximum.
    pub fn tech_price(&self, id: PlayerId, tech: TechId) -> Option<(Cost, u32)> {
        let def = self.config.tech(tech);
        let next = self.tech_level(id, tech) + 1;
        if next > def.max_level {
            return None;
        }
        let mut cost = def.cost;
        let mut ticks = def.research_seconds as u64 * 1000 / self.config.tick_ms.max(1) as u64;
        for _ in 1..next {
            cost = cost.scaled(100 + def.cost_growth_percent as i64, 100);
            // Chaque niveau prend deux fois plus de temps : on ne peut pas tout
            // rechercher dans une partie, il faut choisir sa voie.
            ticks *= 2;
        }
        if next >= 2 {
            cost.stone += def.stone_from_level2 * (next as i64 - 1);
        }
        if next >= 3 {
            cost.iron += def.iron_at_level3;
        }
        Some((cost.to_milli(), ticks as u32))
    }

    /// Vrai si les prérequis (technologie, bâtiment) d'une recherche sont remplis.
    pub fn tech_unlocked(&self, id: PlayerId, tech: TechId) -> bool {
        let Some(p) = self.player(id) else { return false };
        let def = self.config.tech(tech);
        def.requires.is_none_or(|(pre, level)| p.techs[pre.index()] >= level)
            && def.requires_building.is_none_or(|b| p.buildings[b.index()] > 0)
    }

    /// Vérifie qu'une recherche peut être lancée (sans regarder le coût).
    pub fn can_research(&self, id: PlayerId, tech: TechId) -> Result<(), RejectReason> {
        let Some(p) = self.player(id) else {
            return Err(RejectReason::UnknownPlayer);
        };
        if p.buildings[BuildingKind::Academy.index()] == 0 {
            return Err(RejectReason::NoAcademy);
        }
        if p.research.is_some() {
            return Err(RejectReason::Researching);
        }
        let def = self.config.tech(tech);
        if p.techs[tech.index()] >= def.max_level {
            return Err(RejectReason::MaxLevel);
        }
        if !self.tech_unlocked(id, tech) {
            return Err(RejectReason::Locked);
        }
        Ok(())
    }

    pub(crate) fn start_research(&mut self, id: PlayerId, tech: TechId) -> Result<(), RejectReason> {
        if self.phase != Phase::Playing {
            return Err(RejectReason::WrongPhase);
        }
        self.can_research(id, tech)?;
        let Some((cost, ticks)) = self.tech_price(id, tech) else {
            return Err(RejectReason::MaxLevel);
        };
        if !self.player_ref(id).resources.covers(&cost) {
            return Err(RejectReason::NotEnoughResources);
        }
        let level = self.tech_level(id, tech) + 1;
        let p = self.player_mut(id);
        p.resources.pay(&cost);
        p.research = Some(Research {
            tech,
            level,
            remaining: ticks.max(1),
            total: ticks.max(1),
        });
        self.update.events.push(GameEvent::ResearchStarted { player: id, tech, level });
        Ok(())
    }

    /// Fait avancer la recherche en cours d'un joueur.
    pub(crate) fn progress_research(&mut self, id: PlayerId) {
        let p = self.player_mut(id);
        let Some(mut research) = p.research else { return };
        if research.remaining > 1 {
            research.remaining -= 1;
            p.research = Some(research);
            return;
        }
        p.research = None;
        p.techs[research.tech.index()] = research.level;
        self.update.events.push(GameEvent::ResearchCompleted {
            player: id,
            tech: research.tech,
            level: research.level,
        });
    }
}
