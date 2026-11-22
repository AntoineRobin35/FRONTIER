//! Génération procédurale des mondes de FRONTIER.
//!
//! Même graine + mêmes paramètres = même monde, sur toutes les machines
//! (WebAssembly comme natif) : les fonctions mathématiques passent par la
//! crate `libm` et l'aléatoire par `frontier_engine::rng::Rng`.
//!
//! Le générateur reprend le prototype Python (continents déformés, chaînes
//! de montagnes, lacs, climat par latitude, rivières par écoulement) et
//! ajoute des **règles** : un monde qui ne passe pas les contrôles de
//! qualité (trop peu de terres, pas de grand continent) est régénéré.

mod deposits;
mod noise;
mod rivers;

use frontier_engine::rng::{hash2, Rng};
use frontier_engine::{MapData, Regions, Terrain};
use serde::{Deserialize, Serialize};

use noise::value_noise;

/// Largeur de référence du prototype : toutes les tailles de détails sont
/// exprimées pour cette largeur puis mises à l'échelle.
const REFERENCE_WIDTH: f64 = 240.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapSize {
    Small,
    Medium,
    Large,
}

impl MapSize {
    pub fn dimensions(self) -> (u32, u32) {
        match self {
            MapSize::Small => (480, 360),
            MapSize::Medium => (720, 540),
            MapSize::Large => (960, 720),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldParams {
    pub seed: u64,
    pub width: u32,
    pub height: u32,
}

impl WorldParams {
    pub fn new(seed: u64, size: MapSize) -> Self {
        let (width, height) = size.dimensions();
        WorldParams { seed, width, height }
    }
}

/// Indicateurs de qualité d'un monde généré.
#[derive(Clone, Copy, Debug)]
pub struct Quality {
    /// Part des cases franchissables.
    pub land_ratio: f64,
    /// Part de la carte occupée par la plus grande masse terrestre.
    pub largest_region_ratio: f64,
}

impl Quality {
    pub fn assess(map: &MapData) -> Quality {
        let total = map.len() as f64;
        let regions = Regions::compute(map);
        Quality {
            land_ratio: map.passable_count() as f64 / total,
            largest_region_ratio: regions.largest() as f64 / total,
        }
    }

    pub fn is_good(&self) -> bool {
        (0.28..=0.70).contains(&self.land_ratio) && self.largest_region_ratio >= 0.10
    }

    fn score(&self) -> f64 {
        self.largest_region_ratio - (self.land_ratio - 0.40).abs()
    }
}

/// Génère un monde. Si une tentative ne respecte pas les règles de qualité,
/// on réessaie avec une graine dérivée (de façon déterministe).
pub fn generate(params: &WorldParams) -> MapData {
    let mut best: Option<(f64, MapData)> = None;
    for attempt in 0..8u64 {
        let seed = if attempt == 0 {
            params.seed
        } else {
            hash2(params.seed, attempt)
        };
        let mut map = generate_once(params, seed);
        map.seed = params.seed;
        let quality = Quality::assess(&map);
        if quality.is_good() {
            return map;
        }
        let score = quality.score();
        if best.as_ref().is_none_or(|(s, _)| score > *s) {
            best = Some((score, map));
        }
    }
    best.expect("au moins une tentative").1
}

#[derive(Clone, Copy)]
struct Ellipse {
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
}

impl Ellipse {
    #[inline]
    fn shape(&self, x: f64, y: f64) -> f64 {
        let dx = (x - self.cx) / self.rx;
        let dy = (y - self.cy) / self.ry;
        1.0 - (dx * dx + dy * dy).sqrt()
    }
}

struct Segment {
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

impl Segment {
    fn new(a: (f64, f64), b: (f64, f64), reach: f64) -> Self {
        Segment {
            ax: a.0,
            ay: a.1,
            bx: b.0,
            by: b.1,
            min_x: a.0.min(b.0) - reach,
            max_x: a.0.max(b.0) + reach,
            min_y: a.1.min(b.1) - reach,
            max_y: a.1.max(b.1) + reach,
        }
    }

    #[inline]
    fn distance(&self, px: f64, py: f64) -> f64 {
        let dx = self.bx - self.ax;
        let dy = self.by - self.ay;
        let len_sq = dx * dx + dy * dy;
        if len_sq == 0.0 {
            return ((px - self.ax).powi(2) + (py - self.ay).powi(2)).sqrt();
        }
        let t = (((px - self.ax) * dx + (py - self.ay) * dy) / len_sq).clamp(0.0, 1.0);
        let cx = self.ax + t * dx;
        let cy = self.ay + t * dy;
        ((px - cx) * (px - cx) + (py - cy) * (py - cy)).sqrt()
    }
}

fn generate_once(params: &WorldParams, seed: u64) -> MapData {
    let w = params.width as usize;
    let h = params.height as usize;
    let wf = w as f64;
    let hf = h as f64;
    let s = wf / REFERENCE_WIDTH;
    let mut rng = Rng::new(seed);

    // --- Bruits ------------------------------------------------------------
    let coast_noise = value_noise(w, h, 17.0 * s, &mut rng);
    let detail_noise = value_noise(w, h, 7.0 * s, &mut rng);
    let fine_noise = value_noise(w, h, 3.0, &mut rng);
    let moisture_noise = value_noise(w, h, 28.0 * s, &mut rng);
    let warp_x = value_noise(w, h, 34.0 * s, &mut rng);
    let warp_y = value_noise(w, h, 34.0 * s, &mut rng);
    let regional_relief = value_noise(w, h, 32.0 * s, &mut rng);
    let local_relief = value_noise(w, h, 11.0 * s, &mut rng);

    // --- Continents --------------------------------------------------------
    let continent_count = rng.range_i32(3, 7) as usize;
    let mut continents: Vec<Ellipse> = Vec::new();
    let mut attempts = 0;
    while continents.len() < continent_count && attempts < 500 {
        attempts += 1;
        let rx = rng.range_f64(0.085, 0.19) * wf;
        let ry = rng.range_f64(0.12, 0.25) * hf;
        let cx = rng.range_f64(rx + 5.0 * s, wf - rx - 5.0 * s);
        let cy = rng.range_f64(ry + 5.0 * s, hf - ry - 5.0 * s);
        let overlaps = continents.iter().any(|o| {
            let dx = (cx - o.cx) / (rx + o.rx);
            let dy = (cy - o.cy) / (ry + o.ry);
            dx * dx + dy * dy < 0.45
        });
        if !overlaps {
            continents.push(Ellipse { cx, cy, rx, ry });
        }
    }
    while continents.len() < 3 {
        let rx = rng.range_f64(0.10, 0.16) * wf;
        let ry = rng.range_f64(0.14, 0.20) * hf;
        continents.push(Ellipse {
            cx: rng.range_f64(rx, wf - rx),
            cy: rng.range_f64(ry, hf - ry),
            rx,
            ry,
        });
    }

    let islands: Vec<Ellipse> = (0..7)
        .map(|_| Ellipse {
            cx: rng.range_f64(0.08, 0.92) * wf,
            cy: rng.range_f64(0.08, 0.92) * hf,
            rx: rng.range_f64(3.0, 8.0) * s,
            ry: rng.range_f64(4.0, 10.0) * s,
        })
        .collect();

    // --- Chaînes de montagnes ----------------------------------------------
    let ridge_width = 4.5 * s;
    let mut segments = Vec::new();
    for c in &continents {
        let roll = rng.next_f64();
        let chain_count = if roll < 0.10 { 0 } else if roll < 0.75 { 1 } else { 2 };
        for _ in 0..chain_count {
            let angle = rng.range_f64(0.0, std::f64::consts::PI);
            let size = c.rx.min(c.ry);
            let length = size * rng.range_f64(1.0, 1.65);
            let curve = size * rng.range_f64(0.08, 0.30);
            let frequency = rng.range_f64(1.5, 3.5);
            let (sin_a, cos_a) = (libm::sin(angle), libm::cos(angle));

            let mut points = Vec::with_capacity(13);
            for i in 0..13 {
                let t = i as f64 / 12.0 - 0.5;
                let along = t * length;
                let sideways = libm::sin(t * std::f64::consts::PI * frequency) * curve
                    + rng.range_f64(-curve * 0.12, curve * 0.12);
                points.push((
                    c.cx + cos_a * along - sin_a * sideways,
                    c.cy + sin_a * along + cos_a * sideways,
                ));
            }
            for pair in points.windows(2) {
                segments.push(Segment::new(pair[0], pair[1], ridge_width * 3.0));
            }
        }
    }

    // --- Lacs ----------------------------------------------------------------
    let mut lakes = Vec::new();
    for c in &continents {
        if rng.next_f64() < 0.85 {
            lakes.push(Ellipse {
                cx: c.cx + rng.range_f64(-0.35, 0.35) * c.rx,
                cy: c.cy + rng.range_f64(-0.35, 0.35) * c.ry,
                rx: rng.range_f64(3.0, 6.0) * s,
                ry: rng.range_f64(4.0, 8.0) * s,
            });
        }
    }

    // --- Relief, climat, biomes ---------------------------------------------
    let n = w * h;
    let mut elevation = vec![0f32; n];
    let mut moisture = vec![0f32; n];
    let mut terrain = vec![Terrain::DeepWater; n];

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let (xf, yf) = (x as f64, y as f64);

            let sx = xf + (warp_x[i] as f64 - 0.5) * 24.0 * s;
            let sy = yf + (warp_y[i] as f64 - 0.5) * 20.0 * s;

            let mut land = -10.0f64;
            for e in continents.iter().chain(islands.iter()) {
                land = land.max(e.shape(sx, sy));
            }
            land += (coast_noise[i] as f64 - 0.5) * 0.17;
            land += (detail_noise[i] as f64 - 0.5) * 0.06;
            land += (fine_noise[i] as f64 - 0.5) * 0.025;

            let mut is_land = land > 0.0;
            if is_land
                && lakes.iter().any(|l| {
                    let dx = (xf - l.cx) / l.rx;
                    let dy = (yf - l.cy) / l.ry;
                    dx * dx + dy * dy < 1.0
                })
            {
                is_land = false;
            }

            let mut ridge = 0.0f64;
            if is_land {
                for seg in &segments {
                    if xf < seg.min_x || xf > seg.max_x || yf < seg.min_y || yf > seg.max_y {
                        continue;
                    }
                    let d = seg.distance(xf, yf) / ridge_width;
                    ridge = ridge.max(libm::exp(-(d * d)));
                }
            }

            let regional = regional_relief[i] as f64;
            let local = local_relief[i] as f64;

            let height_value = if is_land {
                let interior = (land / 0.55).clamp(0.0, 1.0);
                let plateau_bonus = if regional > 0.72 && ridge < 0.38 { 0.07 } else { 0.0 };
                (0.44
                    + interior * 0.10
                    + (regional - 0.5) * 0.20
                    + (local - 0.5) * 0.055
                    + ridge * 0.32
                    + plateau_bonus)
                    .clamp(0.40, 1.0)
            } else {
                0.0
            };

            let latitude = ((yf / hf) - 0.5).abs() * 2.0;
            let temperature =
                (1.0 - latitude * 0.78 - (height_value - 0.55).max(0.0) * 0.75).clamp(0.0, 1.0);
            let wetness =
                (moisture_noise[i] as f64 + (1.0 - latitude) * 0.08 - ridge * 0.12).clamp(0.0, 1.0);

            terrain[i] = if !is_land {
                if land < -0.12 {
                    Terrain::DeepWater
                } else {
                    Terrain::Water
                }
            } else if land < 0.09 {
                Terrain::Sand
            } else if ridge > 0.64 {
                if temperature < 0.30 {
                    Terrain::SnowPeak
                } else {
                    Terrain::Mountain
                }
            } else if regional > 0.72 && height_value > 0.54 && ridge < 0.35 {
                Terrain::Plateau
            } else if height_value > 0.70 {
                if temperature < 0.30 {
                    Terrain::Snow
                } else {
                    Terrain::Hills
                }
            } else if temperature < 0.18 {
                Terrain::Snow
            } else if wetness > 0.62 {
                Terrain::Forest
            } else if wetness < 0.29 {
                Terrain::DryGrass
            } else {
                Terrain::Grass
            };

            elevation[i] = height_value as f32;
            moisture[i] = wetness as f32;
        }
    }

    // --- Plages : les terres qui touchent l'eau deviennent sableuses --------
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if !matches!(
                terrain[i],
                Terrain::Grass | Terrain::DryGrass | Terrain::Forest | Terrain::Plateau
            ) {
                continue;
            }
            let touches_water = (-1i32..=1).any(|dy| {
                (-1i32..=1).any(|dx| {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    nx >= 0
                        && ny >= 0
                        && (nx as usize) < w
                        && (ny as usize) < h
                        && terrain[ny as usize * w + nx as usize].is_water()
                })
            });
            if touches_water {
                terrain[i] = Terrain::Sand;
            }
        }
    }

    // --- Rivières ----------------------------------------------------------
    let drainage = rivers::compute(&elevation, &terrain, w, h);

    // Vallées : le terrain se creuse et s'humidifie près des rivières.
    let area_scale = s * s;
    let valley_radius = (4.0 * s).round() as i32;
    for i in 0..n {
        if !drainage.is_river[i] {
            continue;
        }
        let strength = (drainage.accumulation[i] as f64 / (120.0 * area_scale)).min(1.0);
        let (x, y) = ((i % w) as i32, (i / w) as i32);
        for dy in -valley_radius..=valley_radius {
            for dx in -valley_radius..=valley_radius {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let ni = ny as usize * w + nx as usize;
                if terrain[ni].is_water() {
                    continue;
                }
                let distance = ((dx * dx + dy * dy) as f64).sqrt();
                if distance > valley_radius as f64 {
                    continue;
                }
                let influence = (1.0 - distance / valley_radius as f64).powi(2);
                elevation[ni] -= (0.035 * strength * influence) as f32;
                moisture[ni] = (moisture[ni] as f64 + 0.18 * strength * influence).min(1.0) as f32;
            }
        }
    }

    // Zones humides le long des cours d'eau.
    for i in 0..n {
        if !drainage.is_river[i] {
            continue;
        }
        let strength = (drainage.accumulation[i] as f64 / (120.0 * area_scale)).min(1.0);
        let radius = ((if strength < 0.5 { 2.0 } else { 3.0 }) * s).round() as i32;
        let (x, y) = ((i % w) as i32, (i / w) as i32);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                if dx * dx + dy * dy > radius * radius {
                    continue;
                }
                let ni = ny as usize * w + nx as usize;
                if matches!(terrain[ni], Terrain::Grass | Terrain::DryGrass) && moisture[ni] >= 0.35 {
                    terrain[ni] = Terrain::Wetland;
                }
            }
        }
    }

    let river = rivers::river_mask_4connected(&drainage, &terrain, w);
    let river_paths = rivers::trace_paths(&drainage, &terrain, w, s);

    let regions = {
        let probe = MapData {
            seed,
            width: params.width,
            height: params.height,
            terrain: terrain.clone(),
            elevation: vec![0; n],
            river: river.clone(),
            rivers: Vec::new(),
            deposit: vec![0; n],
            deposit_sites: Vec::new(),
        };
        Regions::compute(&probe)
    };
    let placed = deposits::place(&terrain, &river, &regions, w, h, s, &mut rng);

    MapData {
        seed,
        width: params.width,
        height: params.height,
        deposit: placed.layer,
        deposit_sites: placed.sites,
        terrain,
        elevation: elevation
            .iter()
            .map(|&e| (e.clamp(0.0, 1.0) as f64 * 65535.0).round() as u16)
            .collect(),
        river,
        rivers: river_paths,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frontier_engine::Deposit;

    #[test]
    fn same_seed_same_world() {
        let params = WorldParams::new(1234, MapSize::Small);
        assert_eq!(generate(&params), generate(&params));
    }

    #[test]
    fn generated_worlds_follow_quality_rules() {
        for seed in [1, 2, 3, 42, 1234] {
            let map = generate(&WorldParams::new(seed, MapSize::Small));
            map.validate().unwrap();
            let q = Quality::assess(&map);
            assert!(q.land_ratio > 0.2, "seed {seed}: trop peu de terres ({q:?})");
            assert!(!map.rivers.is_empty(), "seed {seed}: aucune rivière");
            for kind in [Deposit::Fertile, Deposit::Grove, Deposit::Quarry, Deposit::Iron] {
                let count = map.deposit_sites.iter().filter(|s| s.kind == kind).count();
                assert!(count >= 2, "seed {seed}: trop peu de gisements {kind:?} ({count})");
            }
        }
    }
}
