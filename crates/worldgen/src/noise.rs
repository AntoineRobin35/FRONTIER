use frontier_engine::rng::Rng;

#[inline]
fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Bruit de valeur interpolé : une grille de valeurs aléatoires espacées de
/// `scale` cases, lissée entre les points.
pub fn value_noise(width: usize, height: usize, scale: f64, rng: &mut Rng) -> Vec<f32> {
    let scale = scale.max(1.0);
    let grid_w = libm::ceil(width as f64 / scale) as usize + 2;
    let grid_h = libm::ceil(height as f64 / scale) as usize + 2;
    let grid: Vec<f64> = (0..grid_w * grid_h).map(|_| rng.next_f64()).collect();

    let mut out = Vec::with_capacity(width * height);
    for y in 0..height {
        let gy = y as f64 / scale;
        let iy = gy as usize;
        let fy = smooth(gy - iy as f64);
        for x in 0..width {
            let gx = x as f64 / scale;
            let ix = gx as usize;
            let fx = smooth(gx - ix as f64);

            let a = grid[iy * grid_w + ix];
            let b = grid[iy * grid_w + ix + 1];
            let c = grid[(iy + 1) * grid_w + ix];
            let d = grid[(iy + 1) * grid_w + ix + 1];

            let top = a + (b - a) * fx;
            let bottom = c + (d - c) * fx;
            out.push((top + (bottom - top) * fy) as f32);
        }
    }
    out
}
