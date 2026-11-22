//! Mesure le rythme des parties pour régler l'équilibrage : temps pour
//! occuper les terres libres, passages d'âge, premières éliminations, durée
//! totale. Les IA jouent en difficulté normale sur une carte moyenne.
//!
//! cargo run --release -p frontier-session --example tune -- [graines...]

use frontier_ai::Difficulty;
use frontier_engine::{BuildingKind, GameConfig, GameEvent, Phase};
use frontier_session::{Controller, Seat, Session, SessionSetup};
use frontier_worldgen::MapSize;

const MAX_TICKS: u64 = 40_000; // 66 minutes

fn minutes(tick: Option<u64>) -> String {
    tick.map_or("  -  ".into(), |t| format!("{:>5.1}", t as f64 / 600.0))
}

fn run(name: &str, config: &GameConfig, seed: u64) {
    let agents = ["colon", "conquerant", "opportuniste", "bastion", "conquerant", "colon"];
    let setup = SessionSetup {
        seed,
        map_size: MapSize::Medium,
        seats: agents
            .iter()
            .map(|a| Seat {
                name: a.to_string(),
                controller: Controller::Bot { agent: a.to_string(), difficulty: Difficulty::Normal },
            })
            .collect(),
        config: config.clone(),
    };
    let mut session = Session::new(&setup).expect("session");
    let (mut filled, mut stone, mut iron, mut first_kill) = (None, None, None, None);
    let mut snapshots = Vec::new();
    while session.game().tick_count() < MAX_TICKS {
        let update = session.step();
        let tick = update.tick;
        for event in &update.events {
            match event {
                GameEvent::BuildingCompleted { kind: BuildingKind::Academy, .. } if stone.is_none() => stone = Some(tick),
                GameEvent::ResearchCompleted { level: 3, .. } if iron.is_none() => iron = Some(tick),
                GameEvent::Eliminated { .. } if first_kill.is_none() => first_kill = Some(tick),
                _ => {}
            }
        }
        let game = session.game();
        if filled.is_none() {
            let owned: u32 = game.players().iter().map(|p| p.tiles).sum();
            if owned * 100 >= game.land_tiles() * 90 {
                filled = Some(tick);
            }
        }
        if tick.is_multiple_of(6_000) && tick > 0 {
            let pop: i64 = game.players().iter().map(|p| game.population(p.id)).sum::<i64>() / 1000;
            let alive = game.players().iter().filter(|p| p.alive).count();
            snapshots.push(format!("{}min:pop{}/{}vivants", tick / 600, pop / alive.max(1) as i64, alive));
        }
        if matches!(game.phase(), Phase::Ended { .. }) {
            break;
        }
    }
    let game = session.game();
    let end = matches!(game.phase(), Phase::Ended { .. }).then_some(game.tick_count());
    println!(
        "{name:<12} graine {seed:<5} terres 90% {} | 1re académie {} | 1er niveau 3 {} | 1re élim. {} | fin {} | {}",
        minutes(filled),
        minutes(stone),
        minutes(iron),
        minutes(first_kill),
        minutes(end),
        snapshots.join(" ")
    );
}

fn main() {
    let seeds: Vec<u64> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    let seeds = if seeds.is_empty() { vec![1, 2, 3] } else { seeds };
    let base = GameConfig::default();
    for &seed in &seeds {
        run("défaut", &base, seed);
    }
}
