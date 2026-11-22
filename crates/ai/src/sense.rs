//! Perception : ce qu'une IA a besoin de savoir sur son voisinage.
//! Uniquement des informations visibles par un joueur humain.

use frontier_engine::{Game, PlayerId, TileIdx, NEUTRAL, NO_REGION};

pub struct Surroundings {
    /// Nombre de cases frontalières par voisin, indexé par identifiant
    /// (`0` = terres neutres).
    pub border: Vec<u32>,
    /// Attaques en cours contre moi : (attaquant, puissance en milli).
    pub incoming: Vec<(PlayerId, i64)>,
    /// Mes attaques en cours : (identifiant, cible, soldats en milli).
    pub outgoing: Vec<(u32, PlayerId, i64)>,
    /// Mes cases.
    pub tiles: Vec<TileIdx>,
    /// Masses terrestres où j'ai au moins une case (indexé par région).
    pub my_regions: Vec<bool>,
    /// Mes bateaux en mer.
    pub boats_at_sea: usize,
}

impl Surroundings {
    pub fn neutral_border(&self) -> u32 {
        self.border[NEUTRAL as usize]
    }

    pub fn total_border(&self) -> u32 {
        self.border.iter().sum()
    }

    /// Joueurs voisins vivants.
    pub fn neighbors(&self) -> impl Iterator<Item = PlayerId> + '_ {
        self.border
            .iter()
            .enumerate()
            .skip(1)
            .filter(|(_, &count)| count > 0)
            .map(|(id, _)| id as PlayerId)
    }

    pub fn attacking(&self, target: PlayerId) -> bool {
        self.outgoing.iter().any(|&(_, t, _)| t == target)
    }
}

pub fn sense(game: &Game, me: PlayerId) -> Surroundings {
    let map = game.map();
    let owners = game.owners();
    let regions = game.regions();
    let mut border = vec![0u32; game.players().len() + 1];
    let mut tiles = Vec::new();
    let mut my_regions = vec![false; regions.sizes.len()];

    for tile in 0..map.len() as TileIdx {
        if owners[tile as usize] != me {
            continue;
        }
        tiles.push(tile);
        let region = regions.id[tile as usize];
        if region != NO_REGION {
            my_regions[region as usize] = true;
        }
        for n in map.neighbors(tile) {
            let owner = owners[n as usize];
            if owner != me && map.is_passable(n) {
                border[owner as usize] += 1;
            }
        }
    }

    let incoming = game
        .attacks()
        .iter()
        .filter(|a| a.target == me)
        .map(|a| (a.attacker, game.attack_power(a.attacker, &a.units)))
        .collect();
    let outgoing = game
        .attacks()
        .iter()
        .filter(|a| a.attacker == me)
        .map(|a| (a.id, a.target, a.soldiers()))
        .collect();
    let boats_at_sea = game.boats().iter().filter(|b| b.owner == me).count();

    Surroundings {
        border,
        incoming,
        outgoing,
        tiles,
        my_regions,
        boats_at_sea,
    }
}

/// Voisin le moins défendu (défense totale), s'il y en a un.
pub fn weakest_neighbor(game: &Game, s: &Surroundings) -> Option<PlayerId> {
    s.neighbors().min_by_key(|&id| (game.defense_power(id), id))
}
