//! Placement des gisements : terres fertiles, bosquets, pierre, fer.
//!
//! Règles :
//! - chaque type a son terrain (forêt pour les bosquets, collines pour la
//!   pierre, pied des montagnes pour le fer, plaines près des rivières pour
//!   les terres fertiles) ;
//! - les gisements d'un même type sont espacés : ils sont répartis sur toute
//!   la carte, pas tous au même endroit ;
//! - équité : toute masse terrestre assez grande reçoit au moins des terres
//!   fertiles, un bosquet et de la pierre.

use frontier_engine::rng::Rng;
use frontier_engine::{Deposit, DepositSite, Regions, Terrain, NO_REGION};

/// Espacement entre gisements du même type, à la largeur de référence (240).
const SPACING: [(Deposit, f64); 4] = [
    (Deposit::Fertile, 11.0),
    (Deposit::Grove, 11.0),
    (Deposit::Quarry, 15.0),
    (Deposit::Iron, 22.0),
];

/// Taille d'un gisement (cases).
const CLUSTER: usize = 12;

pub struct Placed {
    pub layer: Vec<u8>,
    pub sites: Vec<DepositSite>,
}

pub fn place(
    terrain: &[Terrain],
    river: &[u8],
    regions: &Regions,
    width: usize,
    height: usize,
    scale: f64,
    rng: &mut Rng,
) -> Placed {
    let n = width * height;
    let near = |i: usize, radius: i32, pred: &dyn Fn(usize) -> bool| -> bool {
        let (x, y) = ((i % width) as i32, (i / width) as i32);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let (nx, ny) = (x + dx, y + dy);
                if nx >= 0 && ny >= 0 && (nx as usize) < width && (ny as usize) < height
                    && pred(ny as usize * width + nx as usize) {
                        return true;
                    }
            }
        }
        false
    };
    let r = (2.0 * scale).round() as i32;
    let eligible = |kind: Deposit, i: usize| -> bool {
        let t = terrain[i];
        if !t.is_passable() {
            return false;
        }
        match kind {
            Deposit::Fertile => {
                matches!(t, Terrain::Grass | Terrain::Wetland) && near(i, r * 2, &|j| river[j] > 0)
            }
            Deposit::Grove => t == Terrain::Forest,
            Deposit::Quarry => {
                matches!(t, Terrain::Hills | Terrain::Plateau)
                    || near(i, r, &|j| matches!(terrain[j], Terrain::Mountain | Terrain::Hills))
            }
            Deposit::Iron => near(i, r, &|j| matches!(terrain[j], Terrain::Mountain | Terrain::SnowPeak)),
            Deposit::None => false,
        }
    };

    // Ordre aléatoire (mais reproductible) des cases candidates.
    let mut order: Vec<u32> = (0..n as u32).collect();
    for i in (1..order.len()).rev() {
        let j = rng.below(i as u32 + 1) as usize;
        order.swap(i, j);
    }

    let mut layer = vec![0u8; n];
    let mut sites: Vec<DepositSite> = Vec::new();
    let far_enough = |sites: &[DepositSite], i: usize, kind: Deposit, spacing: f64| -> bool {
        let (x, y) = ((i % width) as f64, (i / width) as f64);
        sites.iter().all(|s| {
            let (sx, sy) = ((s.tile as usize % width) as f64, (s.tile as usize / width) as f64);
            let d2 = (x - sx).powi(2) + (y - sy).powi(2);
            let min = if s.kind == kind { spacing } else { 3.0 * scale };
            d2 >= min * min
        })
    };

    for (kind, spacing) in SPACING {
        let spacing = spacing * scale;
        for &i in &order {
            let i = i as usize;
            if layer[i] == 0 && regions.size_of(i as u32) >= 300 && eligible(kind, i) && far_enough(&sites, i, kind, spacing) {
                grow_cluster(&mut layer, terrain, i, kind, width, height);
                sites.push(DepositSite { tile: i as u32, kind });
            }
        }
    }

    // Équité : chaque grande masse terrestre a les trois ressources de base.
    let essentials = [Deposit::Fertile, Deposit::Grove, Deposit::Quarry];
    for (region, &size) in regions.sizes.iter().enumerate() {
        if size < 1500 {
            continue;
        }
        for kind in essentials {
            let has = sites
                .iter()
                .any(|s| s.kind == kind && regions.id[s.tile as usize] == region as u32);
            if has {
                continue;
            }
            // À défaut du terrain idéal, n'importe quelle case franchissable.
            let pick = order
                .iter()
                .map(|&i| i as usize)
                .filter(|&i| regions.id[i] == region as u32 && regions.id[i] != NO_REGION && layer[i] == 0)
                .find(|&i| eligible(kind, i))
                .or_else(|| {
                    order
                        .iter()
                        .map(|&i| i as usize)
                        .find(|&i| regions.id[i] == region as u32 && layer[i] == 0)
                });
            if let Some(i) = pick {
                grow_cluster(&mut layer, terrain, i, kind, width, height);
                sites.push(DepositSite { tile: i as u32, kind });
            }
        }
    }

    Placed { layer, sites }
}

/// Marque quelques cases franchissables autour du centre.
fn grow_cluster(layer: &mut [u8], terrain: &[Terrain], center: usize, kind: Deposit, width: usize, height: usize) {
    let mut queue = std::collections::VecDeque::from([center]);
    let mut count = 0;
    layer[center] = kind as u8;
    while let Some(i) = queue.pop_front() {
        count += 1;
        if count >= CLUSTER {
            break;
        }
        let (x, y) = (i % width, i / width);
        let mut next = Vec::with_capacity(4);
        if y > 0 {
            next.push(i - width);
        }
        if x + 1 < width {
            next.push(i + 1);
        }
        if y + 1 < height {
            next.push(i + width);
        }
        if x > 0 {
            next.push(i - 1);
        }
        for j in next {
            if layer[j] == 0 && terrain[j].is_passable() {
                layer[j] = kind as u8;
                queue.push_back(j);
            }
        }
    }
}
