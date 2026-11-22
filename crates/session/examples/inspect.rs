//! Suit l'économie d'un joueur IA au fil du temps (débogage d'équilibrage).
//! cargo run --release -p frontier-session --example inspect -- <graine> <joueur>
use frontier_ai::Difficulty;
use frontier_session::{Controller, Seat, Session, SessionSetup};
use frontier_worldgen::MapSize;
fn main() {
    let mut args = std::env::args().skip(1);
    let seed: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(1);
    let who: u16 = args.next().and_then(|a| a.parse().ok()).unwrap_or(1);
    let agents = ["colon", "conquerant", "opportuniste", "bastion", "conquerant", "colon"];
    let setup = SessionSetup { seed, map_size: MapSize::Medium, seats: agents.iter().map(|a| Seat { name: a.to_string(), controller: Controller::Bot { agent: a.to_string(), difficulty: Difficulty::Normal } }).collect(), config: Default::default() };
    let mut s = Session::new(&setup).unwrap();
    for t in 0..=12_000u64 {
        s.step();
        if t % 1200 == 0 {
            let g = s.game();
            let p = g.player(who).unwrap();
            let k = 1000;
            println!("{:>3}min tiles={:>6} pop={:>5}/{:<5} workers={:>5} jobs={:?} slots={:?} soldiers={:?} gar={} food_prod={} eat={} res=[{} {} {} {}] techs={:?} bld={:?} attacks={}",
                t / 600, p.tiles, g.population(who) / k, g.housing(who) / k, p.workers / k,
                p.jobs.map(|j| j / k), {let sl = g.job_slots(who); [sl.total(frontier_engine::Job::Gatherer)/k, sl.total(frontier_engine::Job::Lumberjack)/k, sl.total(frontier_engine::Job::Miner)/k, sl.total(frontier_engine::Job::Smith)/k]},
                p.soldiers.map(|x| x / k), p.garrison / k, g.production(who).food * 10 / k, g.food_consumption(who) * 10 / k,
                p.resources.food / k, p.resources.wood / k, p.resources.stone / k, p.resources.iron / k, p.techs, p.buildings,
                g.attacks().iter().filter(|a| a.attacker == who).count());
        }
    }
}
