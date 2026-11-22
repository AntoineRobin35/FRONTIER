//! Forme d'un territoire après quelques expéditions, sur de vraies cartes.
//! cargo run --release -p frontier-session --example shape
use frontier_engine::{Game, GameConfig, GameSetup, Intent, PlayerSetup, TileIdx, NEUTRAL};
use frontier_worldgen::{generate, MapSize, WorldParams};
fn main() {
    for seed in 1..=6u64 {
        let map = generate(&WorldParams::new(seed, MapSize::Medium));
        let setup = GameSetup { seed, players: vec![PlayerSetup { name: "J".into() }], config: GameConfig { immunity_ticks: 0, ..GameConfig::default() } };
        let mut game = Game::new(&setup, map).unwrap();
        let (w, h) = (game.map().width, game.map().height);
        let mut spot = None;
        'f: for dy in 0..h / 2 {
            for dx in 0..w / 2 {
                let t = game.map().idx(w / 2 + dx, h / 2 + dy);
                if game.map().is_passable(t) { spot = Some(t); break 'f; }
            }
        }
        game.tick(&[(1, Intent::Spawn { tile: spot.unwrap() })]);
        for _ in 0..4 {
            game.tick(&[(1, Intent::Attack { target: NEUTRAL, ratio_permille: 600 })]);
            for _ in 0..150 { game.tick(&[]); }
        }
        let (mut x0, mut x1, mut y0, mut y1, mut n) = (u32::MAX, 0, u32::MAX, 0, 0);
        let mut terr = [0u32; 12];
        for t in 0..game.map().len() as TileIdx {
            if game.owner(t) == 1 {
                let (x, y) = game.map().xy(t);
                x0 = x0.min(x); x1 = x1.max(x); y0 = y0.min(y); y1 = y1.max(y); n += 1;
                terr[game.map().terrain(t) as usize] += 1;
            }
        }
        println!("graine {seed} : {n} cases, {} × {} (largeur × hauteur), terrains {:?}", x1 - x0 + 1, y1 - y0 + 1, terr);
    }
}
