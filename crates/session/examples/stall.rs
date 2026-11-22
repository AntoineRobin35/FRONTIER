//! Pourquoi les IA cessent d'attaquer : état de chaque joueur à une minute donnée.
//! cargo run --release -p frontier-session --example stall -- <graine> <minute>
use frontier_ai::{sense::sense, BotCore, Difficulty};
use frontier_session::{Controller, Seat, Session, SessionSetup};
use frontier_worldgen::MapSize;
fn main() {
    let mut args = std::env::args().skip(1);
    let seed: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(3);
    let minute: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(40);
    let agents = ["colon", "conquerant", "opportuniste", "bastion", "conquerant", "colon"];
    let setup = SessionSetup { seed, map_size: MapSize::Medium, seats: agents.iter().map(|a| Seat { name: a.to_string(), controller: Controller::Bot { agent: a.to_string(), difficulty: Difficulty::Normal } }).collect(), config: Default::default() };
    let mut s = Session::new(&setup).unwrap();
    while s.game().tick_count() < minute * 600 {
        s.step();
    }
    let g = s.game();
    for p in g.players().iter().filter(|p| p.alive) {
        let sur = sense(g, p.id);
        let n: Vec<String> = sur.neighbors().map(|id| format!("{id}(déf {})", g.defense_power(id) / 1000)).collect();
        println!(
            "{} {:<12} cases {:>6} pop {:>5}/{:<5} soldats {:?} garnison {} x{} | frappe60% {} défense {} | voisins {:?} attaques {}",
            p.id, p.name, p.tiles, g.population(p.id) / 1000, g.housing(p.id) / 1000,
            p.soldiers.iter().map(|x| x / 1000).collect::<Vec<_>>(), p.garrison / 1000, p.garrison_training,
            BotCore::strike_power(g, p.id, 600) / 1000, g.defense_power(p.id) / 1000, n,
            g.attacks().iter().filter(|a| a.attacker == p.id).count()
        );
    }
}
