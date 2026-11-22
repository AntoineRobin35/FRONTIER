//! IA de base. Chacune tient dans un fichier court et sert d'exemple.

mod bastion;
mod colon;
mod conquerant;
mod opportuniste;
mod passif;

pub use bastion::Bastion;
pub use colon::Colon;
pub use conquerant::Conquerant;
pub use opportuniste::Opportuniste;
pub use passif::Passif;

use frontier_engine::{Game, Intent, PlayerId, NEUTRAL};

use crate::sense::Surroundings;
use crate::BotCore;

pub(crate) fn attack(out: &mut Vec<Intent>, target: PlayerId, ratio_permille: u16) {
    out.push(Intent::Attack { target, ratio_permille });
}

/// Colonise les terres libres voisines si l'armée le permet.
pub(crate) fn expand(core: &BotCore, game: &Game, me: PlayerId, s: &Surroundings, ratio: u16, out: &mut Vec<Intent>) -> bool {
    let army = game.player(me).map_or(0, |p| p.army());
    if s.neutral_border() > 0 && !s.attacking(NEUTRAL) && army >= 20_000 {
        attack(out, NEUTRAL, core.ratio(ratio));
        return true;
    }
    false
}

/// Attaque le voisin le moins défendu si notre frappe le dépasse nettement.
/// `margin` : rapport exigé (‰), par exemple 1500 = 1,5 fois sa défense.
pub(crate) fn strike_weakest(
    core: &BotCore,
    game: &Game,
    me: PlayerId,
    s: &Surroundings,
    ratio: u16,
    margin: i64,
    out: &mut Vec<Intent>,
) -> bool {
    if game.in_immunity() {
        return false;
    }
    let ratio = core.ratio(ratio);
    let mine = BotCore::strike_power(game, me, ratio);
    // Plus la partie avance, plus on ose : -1,5 % d'exigence par minute
    // après 15 minutes, jusqu'à -60 %.
    let minutes = game.started_at().map_or(0, |s| game.tick_count().saturating_sub(s) / 600) as i64;
    let margin = margin * (100 - ((minutes - 15) * 3 / 2).clamp(0, 60)) / 100;
    let target = s
        .neighbors()
        .filter(|&id| !s.attacking(id))
        .min_by_key(|&id| (game.defense_power(id), id));
    if let Some(target) = target {
        if mine * 1000 > game.defense_power(target) * margin {
            attack(out, target, ratio);
            return true;
        }
    }
    false
}

/// Riposte contre l'attaque la plus forte si on peut la contrer.
pub(crate) fn retaliate(core: &BotCore, game: &Game, me: PlayerId, s: &Surroundings, out: &mut Vec<Intent>) -> Option<PlayerId> {
    let &(enemy, power) = s.incoming.iter().max_by_key(|x| x.1)?;
    if !s.attacking(enemy) && BotCore::strike_power(game, me, core.ratio(500)) > power {
        attack(out, enemy, core.ratio(500));
    }
    Some(enemy)
}
