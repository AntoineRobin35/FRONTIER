//! Parties complètes entre IA : stabilité et déterminisme de bout en bout.

use frontier_ai::Difficulty;
use frontier_session::{Controller, Seat, Session, SessionSetup};
use frontier_worldgen::MapSize;

fn setup(seed: u64) -> SessionSetup {
    let agents = ["colon", "conquerant", "opportuniste", "bastion", "passif"];
    SessionSetup {
        seed,
        map_size: MapSize::Small,
        seats: agents
            .iter()
            .map(|a| Seat {
                name: a.to_string(),
                controller: Controller::Bot {
                    agent: a.to_string(),
                    difficulty: Difficulty::Normal,
                },
            })
            .collect(),
        config: Default::default(),
    }
}

fn run(seed: u64, ticks: u64) -> (u64, Session) {
    let mut session = Session::new(&setup(seed)).unwrap();
    for _ in 0..ticks {
        session.step();
    }
    (session.game().state_hash(), session)
}

#[test]
fn bot_game_is_deterministic() {
    let (a, _) = run(7, 1_500);
    let (b, _) = run(7, 1_500);
    assert_eq!(a, b);
}

#[test]
fn bots_expand_early() {
    for seed in [1, 2, 3] {
        let (_, session) = run(seed, 3_000);
        let game = session.game();
        let expanding = game.players().iter().filter(|p| p.tiles > 1_000).count();
        assert!(expanding >= 3, "seed {seed}: les IA ne s'étendent pas");
    }
}

#[test]
fn bot_game_stays_consistent() {
    for seed in [1, 2, 3] {
        let (_, session) = run(seed, 3_000);
        let game = session.game();
        assert!(game.players().iter().all(|p| p.spawned), "seed {seed}: placement incomplet");
        for p in game.players() {
            let owned = game.owners().iter().filter(|&&o| o == p.id).count() as u32;
            assert_eq!(owned, p.tiles, "seed {seed}: cases incohérentes pour {}", p.name);
            assert!(p.workers >= 0 && p.garrison >= 0 && p.soldiers.iter().all(|&x| x >= 0));
            assert!(game.population(p.id) <= game.housing(p.id) + 1000 || !p.alive);
            let r = p.resources;
            assert!(r.food >= 0 && r.wood >= 0 && r.stone >= 0 && r.iron >= 0, "ressources négatives : {r:?}");
            assert_eq!(p.alive, p.tiles > 0, "seed {seed}: état de vie incohérent");
        }
    }
}
