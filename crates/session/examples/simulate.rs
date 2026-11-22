//! Simulation sans affichage : fait s'affronter des IA et affiche le
//! résultat. Utile pour équilibrer le jeu et comparer des IA.
//!
//! cargo run --release -p frontier-session --example simulate -- 42 12000

use frontier_ai::Difficulty;
use frontier_engine::{GameEvent, Phase};
use frontier_session::{Controller, Seat, Session, SessionSetup};
use frontier_worldgen::MapSize;

fn main() {
    let mut args = std::env::args().skip(1);
    let seed: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(42);
    let max_ticks: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(12_000);

    let agents = ["colon", "conquerant", "opportuniste", "bastion", "conquerant", "colon"];
    let setup = SessionSetup {
        seed,
        map_size: MapSize::Medium,
        seats: agents
            .iter()
            .enumerate()
            .map(|(i, a)| Seat {
                name: format!("{a}-{}", i + 1),
                controller: Controller::Bot {
                    agent: a.to_string(),
                    difficulty: Difficulty::Normal,
                },
            })
            .collect(),
        config: Default::default(),
    };

    let start = std::time::Instant::now();
    let mut session = Session::new(&setup).expect("session");
    println!("Monde généré en {:?}", start.elapsed());

    let start = std::time::Instant::now();
    let mut eliminations = Vec::new();
    let mut boats = 0;
    while session.game().tick_count() < max_ticks {
        let update = session.step();
        for event in &update.events {
            match event {
                GameEvent::Eliminated { player, by } => eliminations.push((update.tick, *player, *by)),
                GameEvent::BoatLaunched { .. } => boats += 1,
                GameEvent::ResearchCompleted { player, tech, level } => {
                    println!("  tour {:>6} : joueur {player} recherche {tech:?} niveau {level}", update.tick)
                }
                _ => {}
            }
        }
        if matches!(session.game().phase(), Phase::Ended { .. }) {
            break;
        }
    }
    let game = session.game();
    println!(
        "{} tours ({:.0} s de jeu) en {:?}, phase : {:?}",
        game.tick_count(),
        game.tick_count() as f64 / 10.0,
        start.elapsed(),
        game.phase()
    );
    println!("  bateaux lancés : {boats}");
    for (tick, player, by) in eliminations {
        println!("  tour {tick:>6} : joueur {player} éliminé par {by:?}");
    }
    for p in game.players() {
        println!(
            "  {:<16} vivant={:<5} cases={:>6} ({:>4.1}%) pop={:>6}/{:<6} soldats={:>5} garnison={:>5} techs={:?} bâtiments={:?}",
            p.name,
            p.alive,
            p.tiles,
            p.tiles as f64 * 100.0 / game.land_tiles() as f64,
            game.population(p.id) / 1000,
            game.housing(p.id) / 1000,
            p.army() / 1000,
            p.garrison / 1000,
            p.techs,
            p.buildings
        );
    }
}
