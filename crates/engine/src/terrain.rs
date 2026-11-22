use serde::{Deserialize, Serialize};

/// Type de terrain d'une case.
///
/// La valeur numérique est stable : elle est utilisée dans les fichiers de
/// carte, par le rendu (palette de couleurs) et par les observations des IA.
/// On peut ajouter des terrains à la fin, jamais réordonner.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Terrain {
    DeepWater = 0,
    Water = 1,
    Sand = 2,
    Grass = 3,
    DryGrass = 4,
    Forest = 5,
    Wetland = 6,
    Plateau = 7,
    Hills = 8,
    Mountain = 9,
    SnowPeak = 10,
    Snow = 11,
}

impl Terrain {
    pub const COUNT: usize = 12;

    pub const ALL: [Terrain; Terrain::COUNT] = [
        Terrain::DeepWater,
        Terrain::Water,
        Terrain::Sand,
        Terrain::Grass,
        Terrain::DryGrass,
        Terrain::Forest,
        Terrain::Wetland,
        Terrain::Plateau,
        Terrain::Hills,
        Terrain::Mountain,
        Terrain::SnowPeak,
        Terrain::Snow,
    ];

    pub fn from_u8(value: u8) -> Option<Terrain> {
        Self::ALL.get(value as usize).copied()
    }

    #[inline]
    pub fn is_water(self) -> bool {
        matches!(self, Terrain::DeepWater | Terrain::Water)
    }

    /// Une case franchissable peut être possédée et conquise.
    #[inline]
    pub fn is_passable(self) -> bool {
        !self.is_water() && !matches!(self, Terrain::Mountain | Terrain::SnowPeak)
    }

    pub fn label_fr(self) -> &'static str {
        match self {
            Terrain::DeepWater => "Océan",
            Terrain::Water => "Eaux peu profondes",
            Terrain::Sand => "Plage",
            Terrain::Grass => "Prairie",
            Terrain::DryGrass => "Steppe",
            Terrain::Forest => "Forêt",
            Terrain::Wetland => "Marais",
            Terrain::Plateau => "Plateau",
            Terrain::Hills => "Collines",
            Terrain::Mountain => "Montagne",
            Terrain::SnowPeak => "Sommet enneigé",
            Terrain::Snow => "Toundra",
        }
    }
}
