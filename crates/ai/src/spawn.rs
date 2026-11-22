use frontier_engine::rng::Rng;
use frontier_engine::{Game, TileIdx, NEUTRAL};

/// Choisit un point de départ sur une grande masse terrestre, le plus loin
/// possible des joueurs déjà placés. Évite les îlots.
pub fn pick_spawn(game: &Game, rng: &mut Rng) -> Option<TileIdx> {
    let map = game.map();
    let regions = game.regions();
    let min_region = (game.land_tiles() / 20).max(200).min(regions.largest());

    let others: Vec<(i64, i64)> = game
        .players()
        .iter()
        .filter_map(|p| p.spawn_tile)
        .map(|t| {
            let (x, y) = map.xy(t);
            (x as i64, y as i64)
        })
        .collect();

    let mut best: Option<(i64, TileIdx)> = None;
    for _ in 0..400 {
        let tile = rng.below(map.len() as u32);
        if !map.is_passable(tile)
            || game.owner(tile) != NEUTRAL
            || regions.size_of(tile) < min_region
        {
            continue;
        }
        let (x, y) = map.xy(tile);
        let (x, y) = (x as i64, y as i64);
        let distance = others
            .iter()
            .map(|&(ox, oy)| (x - ox).pow(2) + (y - oy).pow(2))
            .min()
            .unwrap_or(i64::MAX / 2);
        // Préfère aussi les terres fertiles au voisinage immédiat.
        let fertile = map
            .neighbors(tile)
            .filter(|&n| map.is_passable(n))
            .count() as i64;
        let score = distance + fertile;
        if best.is_none_or(|(s, _)| score > s) {
            best = Some((score, tile));
        }
    }
    best.map(|(_, tile)| tile)
}
