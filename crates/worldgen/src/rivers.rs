//! Réseau hydrographique : écoulement de l'eau, accumulation, ordre de
//! Strahler, puis tracés lissés pour l'affichage.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use frontier_engine::{RiverPath, Terrain};

pub const NONE: u32 = u32::MAX;

const DIRS8: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

pub struct Drainage {
    /// Case vers laquelle l'eau s'écoule (`NONE` pour l'eau libre).
    pub flow_to: Vec<u32>,
    /// Nombre de cases dont l'eau passe par ici.
    pub accumulation: Vec<u32>,
    /// Ordre de Strahler (hiérarchie des cours d'eau).
    pub order: Vec<u8>,
    pub is_river: Vec<bool>,
}

/// Calcule l'écoulement par « priority flood » : on remplit virtuellement
/// les cuvettes depuis les mers et les lacs, ce qui garantit que chaque
/// goutte d'eau finit dans une étendue d'eau.
pub fn compute(elevation: &[f32], terrain: &[Terrain], width: usize, height: usize) -> Drainage {
    let n = width * height;
    let mut flow_to = vec![NONE; n];
    let mut visited = vec![false; n];
    let mut heap = BinaryHeap::new();
    let mut order = Vec::with_capacity(n);

    for i in 0..n {
        if terrain[i].is_water() {
            visited[i] = true;
            // Les altitudes sont positives : l'ordre des bits suit l'ordre
            // des valeurs, ce qui donne une clé entière déterministe.
            heap.push(Reverse((elevation[i].max(0.0).to_bits(), i as u32)));
        }
    }

    while let Some(Reverse((key, i))) = heap.pop() {
        let current = f32::from_bits(key);
        order.push(i);
        let x = i as usize % width;
        let y = i as usize / width;
        for (dx, dy) in DIRS8 {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                continue;
            }
            let ni = ny as usize * width + nx as usize;
            if visited[ni] {
                continue;
            }
            visited[ni] = true;
            let filled = elevation[ni].max(current + 1e-6);
            flow_to[ni] = i;
            heap.push(Reverse((filled.to_bits(), ni as u32)));
        }
    }

    // Accumulation : chaque case transmet son eau en aval.
    let mut accumulation: Vec<u32> = terrain
        .iter()
        .map(|t| if t.is_water() { 0 } else { 1 })
        .collect();
    for &i in order.iter().rev() {
        let parent = flow_to[i as usize];
        if parent != NONE {
            accumulation[parent as usize] += accumulation[i as usize];
        }
    }

    // Ordre de Strahler.
    let mut max_upstream = vec![0u8; n];
    let mut count_max = vec![0u8; n];
    let mut stream_order = vec![0u8; n];
    for &i in order.iter().rev() {
        let i = i as usize;
        if terrain[i].is_water() {
            continue;
        }
        let maximum = max_upstream[i];
        let current = if maximum == 0 {
            1
        } else if count_max[i] >= 2 {
            maximum.saturating_add(1)
        } else {
            maximum
        };
        stream_order[i] = current;

        let parent = flow_to[i];
        if parent == NONE || terrain[parent as usize].is_water() {
            continue;
        }
        let p = parent as usize;
        if current > max_upstream[p] {
            max_upstream[p] = current;
            count_max[p] = 1;
        } else if current == max_upstream[p] {
            count_max[p] = count_max[p].saturating_add(1);
        }
    }

    // Sélection des cours d'eau principaux.
    let threshold = ((n as f64 * 0.002) as u32).max(80);
    let mut is_river = vec![false; n];
    for i in 0..n {
        if terrain[i].is_water() {
            continue;
        }
        let flow = accumulation[i];
        if flow >= threshold && (stream_order[i] >= 2 || flow >= threshold * 2) {
            is_river[i] = true;
        }
    }

    Drainage {
        flow_to,
        accumulation,
        order: stream_order,
        is_river,
    }
}

/// Bouche les passages en diagonale : une attaque qui avance case par case
/// (4 directions) ne doit pas pouvoir traverser une rivière sans la toucher.
pub fn river_mask_4connected(d: &Drainage, terrain: &[Terrain], width: usize) -> Vec<u8> {
    let n = d.is_river.len();
    let mut mask = vec![0u8; n];
    for i in 0..n {
        if !d.is_river[i] {
            continue;
        }
        mask[i] = d.order[i].max(1);
        let t = d.flow_to[i];
        if t == NONE {
            continue;
        }
        let (x, y) = (i % width, i / width);
        let (tx, ty) = (t as usize % width, t as usize / width);
        if x != tx && y != ty {
            let corner = y * width + tx;
            if !terrain[corner].is_water() && mask[corner] == 0 {
                mask[corner] = d.order[i].max(1);
            }
        }
    }
    mask
}

/// Tracés lissés des rivières pour le rendu, en unités de case.
pub fn trace_paths(
    d: &Drainage,
    terrain: &[Terrain],
    width: usize,
    scale: f64,
) -> Vec<RiverPath> {
    let n = terrain.len();

    let mut upstream = vec![0u8; n];
    for i in 0..n {
        if !d.is_river[i] {
            continue;
        }
        let t = d.flow_to[i];
        if t != NONE && d.is_river[t as usize] {
            upstream[t as usize] = upstream[t as usize].saturating_add(1);
        }
    }

    let width_of = |order: u8| -> f64 { (0.12 + order as f64 * 0.16).min(1.3) * scale * 0.7 };

    let mut paths = Vec::new();
    for start in 0..n {
        if !d.is_river[start] || upstream[start] == 1 {
            continue;
        }

        let mut tiles = vec![start];
        let mut current = start;
        let mut mouth = None;
        loop {
            let t = d.flow_to[current];
            if t == NONE {
                break;
            }
            let t = t as usize;
            if terrain[t].is_water() {
                let (cx, cy) = ((current % width) as f64, (current / width) as f64);
                let (tx, ty) = ((t % width) as f64, (t / width) as f64);
                mouth = Some(((cx + tx + 1.0) / 2.0, (cy + ty + 1.0) / 2.0));
                break;
            }
            if !d.is_river[t] {
                break;
            }
            tiles.push(t);
            if upstream[t] != 1 {
                break; // confluence
            }
            current = t;
        }

        let mut points: Vec<(f64, f64, f64)> = tiles
            .iter()
            .map(|&i| {
                (
                    (i % width) as f64 + 0.5,
                    (i / width) as f64 + 0.5,
                    width_of(d.order[i]),
                )
            })
            .collect();
        if let (Some((mx, my)), Some(&(_, _, w))) = (mouth, points.last()) {
            points.push((mx, my, w));
        }
        if points.len() < 2 {
            continue;
        }

        let points = smooth(&meander(&points, scale), 2);
        paths.push(RiverPath {
            points: points
                .into_iter()
                .map(|(x, y, w)| [x as f32, y as f32, w as f32])
                .collect(),
        });
    }

    paths
}

/// Légère ondulation des cours d'eau, atténuée aux extrémités.
fn meander(points: &[(f64, f64, f64)], scale: f64) -> Vec<(f64, f64, f64)> {
    if points.len() < 4 {
        return points.to_vec();
    }

    let mut distances = vec![0.0];
    for i in 1..points.len() {
        let dx = points[i].0 - points[i - 1].0;
        let dy = points[i].1 - points[i - 1].1;
        distances.push(distances[i - 1] + (dx * dx + dy * dy).sqrt());
    }
    let total = *distances.last().unwrap();
    let taper_len = 5.0 * scale;

    let mut out = Vec::with_capacity(points.len());
    for (i, &(x, y, w)) in points.iter().enumerate() {
        if i == 0 || i == points.len() - 1 {
            out.push((x, y, w));
            continue;
        }
        let dx = points[i + 1].0 - points[i - 1].0;
        let dy = points[i + 1].1 - points[i - 1].1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-4 {
            out.push((x, y, w));
            continue;
        }
        let (nx, ny) = (-dy / len, dx / len);
        let dist = distances[i];
        let taper = 1.0f64.min(dist / taper_len).min((total - dist) / taper_len);
        let amplitude = (0.9f64).min(w * 1.2);
        let wave = libm::sin(dist / (9.4 * scale)) * 0.65 + libm::sin(dist / (4.6 * scale) + 1.7) * 0.25;
        let offset = wave * amplitude * taper;
        out.push((x + nx * offset, y + ny * offset, w));
    }
    out
}

/// Lissage de Chaikin en gardant les extrémités.
fn smooth(points: &[(f64, f64, f64)], iterations: usize) -> Vec<(f64, f64, f64)> {
    let mut points = points.to_vec();
    if points.len() < 3 {
        return points;
    }
    for _ in 0..iterations {
        let mut out = Vec::with_capacity(points.len() * 2);
        out.push(points[0]);
        for pair in points.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            out.push((
                a.0 * 0.75 + b.0 * 0.25,
                a.1 * 0.75 + b.1 * 0.25,
                a.2 * 0.75 + b.2 * 0.25,
            ));
            out.push((
                a.0 * 0.25 + b.0 * 0.75,
                a.1 * 0.25 + b.1 * 0.75,
                a.2 * 0.25 + b.2 * 0.75,
            ));
        }
        out.push(*points.last().unwrap());
        points = out;
    }
    points
}
