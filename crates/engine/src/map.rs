//! Données de carte : terrain, relief, rivières.
//!
//! Une carte est une donnée, pas un algorithme : elle peut venir du
//! générateur procédural (`frontier-worldgen`), d'un fichier fait à la main,
//! ou être envoyée par le serveur. Le moteur ne fait que la lire.
//!
//! Toutes les grilles sont stockées à plat, ligne par ligne :
//! `index = y * width + x`.

use serde::{Deserialize, Serialize};

use crate::defs::Deposit;
use crate::terrain::Terrain;

/// Index d'une case dans les grilles à plat.
pub type TileIdx = u32;

/// Tracé d'une rivière, uniquement pour l'affichage.
/// Chaque point vaut `[x, y, largeur]`, en unités de case.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiverPath {
    pub points: Vec<[f32; 3]>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapData {
    /// Graine d'origine (affichage, rejouabilité).
    pub seed: u64,
    pub width: u32,
    pub height: u32,
    pub terrain: Vec<Terrain>,
    /// Altitude normalisée, 0 = niveau de la mer, 65535 = sommet.
    pub elevation: Vec<u16>,
    /// 0 = pas de rivière, sinon ordre de Strahler du cours d'eau.
    /// Utilisé par la simulation (franchir une rivière coûte plus cher).
    pub river: Vec<u8>,
    /// Tracés lissés pour le rendu (non utilisés par la simulation).
    pub rivers: Vec<RiverPath>,
    /// Gisement de chaque case (`Deposit as u8`, 0 = aucun).
    #[serde(default)]
    pub deposit: Vec<u8>,
    /// Centre de chaque gisement (pour les icônes et les IA).
    #[serde(default)]
    pub deposit_sites: Vec<DepositSite>,
}

/// Un gisement : quelques cases du même type autour d'un centre.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositSite {
    pub tile: TileIdx,
    pub kind: Deposit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapError {
    BadDimensions,
    LayerSize(&'static str),
}

impl std::fmt::Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapError::BadDimensions => write!(f, "dimensions de carte invalides"),
            MapError::LayerSize(layer) => write!(f, "taille incorrecte pour la couche `{layer}`"),
        }
    }
}

impl std::error::Error for MapError {}

/// Taille maximale d'un côté de carte (garde-fou mémoire).
pub const MAX_MAP_SIDE: u32 = 4096;

impl MapData {
    pub fn validate(&self) -> Result<(), MapError> {
        if self.width == 0
            || self.height == 0
            || self.width > MAX_MAP_SIDE
            || self.height > MAX_MAP_SIDE
        {
            return Err(MapError::BadDimensions);
        }
        let n = self.len();
        if self.terrain.len() != n {
            return Err(MapError::LayerSize("terrain"));
        }
        if self.elevation.len() != n {
            return Err(MapError::LayerSize("elevation"));
        }
        if self.river.len() != n {
            return Err(MapError::LayerSize("river"));
        }
        if self.deposit.len() != n {
            return Err(MapError::LayerSize("deposit"));
        }
        Ok(())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.width as usize * self.height as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn idx(&self, x: u32, y: u32) -> TileIdx {
        y * self.width + x
    }

    #[inline]
    pub fn xy(&self, tile: TileIdx) -> (u32, u32) {
        (tile % self.width, tile / self.width)
    }

    #[inline]
    pub fn terrain(&self, tile: TileIdx) -> Terrain {
        self.terrain[tile as usize]
    }

    #[inline]
    pub fn is_passable(&self, tile: TileIdx) -> bool {
        self.terrain[tile as usize].is_passable()
    }

    #[inline]
    pub fn deposit(&self, tile: TileIdx) -> Deposit {
        Deposit::from_u8(self.deposit[tile as usize])
    }

    #[inline]
    pub fn has_river(&self, tile: TileIdx) -> bool {
        self.river[tile as usize] > 0
    }

    /// Les 4 voisines (haut, droite, bas, gauche), dans un ordre fixe.
    #[inline]
    pub fn neighbors(&self, tile: TileIdx) -> Neighbors {
        let (x, y) = self.xy(tile);
        let w = self.width;
        let mut n = Neighbors { buf: [0; 4], len: 0, pos: 0 };
        if y > 0 {
            n.push(tile - w);
        }
        if x + 1 < w {
            n.push(tile + 1);
        }
        if y + 1 < self.height {
            n.push(tile + w);
        }
        if x > 0 {
            n.push(tile - 1);
        }
        n
    }

    /// Nombre de cases franchissables.
    pub fn passable_count(&self) -> u32 {
        self.terrain.iter().filter(|t| t.is_passable()).count() as u32
    }
}

/// Itérateur sur les voisines d'une case, sans allocation.
pub struct Neighbors {
    buf: [TileIdx; 4],
    len: u8,
    pos: u8,
}

impl Neighbors {
    #[inline]
    fn push(&mut self, tile: TileIdx) {
        self.buf[self.len as usize] = tile;
        self.len += 1;
    }
}

impl Iterator for Neighbors {
    type Item = TileIdx;

    #[inline]
    fn next(&mut self) -> Option<TileIdx> {
        if self.pos < self.len {
            let tile = self.buf[self.pos as usize];
            self.pos += 1;
            Some(tile)
        } else {
            None
        }
    }
}

pub const NO_REGION: u32 = u32::MAX;

/// Masses terrestres connexes (continents, îles).
#[derive(Clone, Debug)]
pub struct Regions {
    /// Région de chaque case, `NO_REGION` pour les cases infranchissables.
    pub id: Vec<u32>,
    /// Nombre de cases de chaque région.
    pub sizes: Vec<u32>,
}

impl Regions {
    pub fn compute(map: &MapData) -> Regions {
        let n = map.len();
        let mut id = vec![NO_REGION; n];
        let mut sizes = Vec::new();
        let mut stack = Vec::new();

        for start in 0..n as TileIdx {
            if id[start as usize] != NO_REGION || !map.is_passable(start) {
                continue;
            }
            let region = sizes.len() as u32;
            let mut size = 0u32;
            id[start as usize] = region;
            stack.push(start);
            while let Some(tile) = stack.pop() {
                size += 1;
                for next in map.neighbors(tile) {
                    if id[next as usize] == NO_REGION && map.is_passable(next) {
                        id[next as usize] = region;
                        stack.push(next);
                    }
                }
            }
            sizes.push(size);
        }

        Regions { id, sizes }
    }

    /// Taille de la région d'une case (0 si infranchissable).
    #[inline]
    pub fn size_of(&self, tile: TileIdx) -> u32 {
        match self.id[tile as usize] {
            NO_REGION => 0,
            region => self.sizes[region as usize],
        }
    }

    pub fn largest(&self) -> u32 {
        self.sizes.iter().copied().max().unwrap_or(0)
    }
}
