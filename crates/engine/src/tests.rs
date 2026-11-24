//! Tests du moteur sur de petites cartes faites à la main.

use crate::*;

/// Prairie entourée d'eau. `channel` : colonnes d'eau qui coupent la carte
/// en deux îles.
fn test_map(width: u32, height: u32, channel: Option<(u32, u32)>) -> MapData {
    let mut terrain = vec![Terrain::Grass; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            let border = x == 0 || y == 0 || x == width - 1 || y == height - 1;
            let water = channel.is_some_and(|(a, b)| x >= a && x <= b);
            if border || water {
                terrain[(y * width + x) as usize] = Terrain::DeepWater;
            }
        }
    }
    let n = terrain.len();
    MapData {
        seed: 1,
        width,
        height,
        terrain,
        elevation: vec![30_000; n],
        river: vec![0; n],
        rivers: Vec::new(),
        deposit: vec![0; n],
        deposit_sites: Vec::new(),
    }
}

fn config() -> GameConfig {
    GameConfig {
        immunity_ticks: 0,
        ..GameConfig::default()
    }
}

fn start_with(map: MapData, spawns: &[(u32, u32)], config: GameConfig) -> Game {
    let setup = GameSetup {
        seed: 9,
        players: (0..spawns.len()).map(|i| PlayerSetup { name: format!("J{i}") }).collect(),
        config,
    };
    let mut game = Game::new(&setup, map).unwrap();
    let intents: Vec<_> = spawns
        .iter()
        .enumerate()
        .map(|(i, &(x, y))| (i as PlayerId + 1, Intent::Spawn { tile: game.map().idx(x, y) }))
        .collect();
    game.tick(&intents);
    assert_eq!(*game.phase(), Phase::Playing);
    game
}

fn start(map: MapData, spawns: &[(u32, u32)]) -> Game {
    start_with(map, spawns, config())
}

fn start_two_players() -> Game {
    start(test_map(60, 30, None), &[(10, 15), (50, 15)])
}

fn run(game: &mut Game, ticks: u32) {
    for _ in 0..ticks {
        game.tick(&[]);
    }
}

fn rich(game: &mut Game, id: PlayerId) {
    game.player_mut(id).resources = Cost::new(1_000_000, 1_000_000, 1_000_000, 1_000_000).to_milli();
}

fn soldiers(game: &Game, id: PlayerId) -> i64 {
    game.player(id).unwrap().army()
}

fn rejected(game: &Game, player: PlayerId, reason: RejectReason) -> bool {
    game.last_update().events.contains(&GameEvent::Rejected { player, reason })
}

/// Fait se toucher les deux joueurs : toutes les cases de x ≤ milieu au
/// joueur 1, les autres au joueur 2.
fn split_map(game: &mut Game) {
    let w = game.map().width;
    for tile in 0..game.map().len() as TileIdx {
        if !game.map().is_passable(tile) {
            continue;
        }
        let owner = if game.map().xy(tile).0 < w / 2 { 1 } else { 2 };
        game.set_owner(tile, owner);
    }
}

// --- Placement -----------------------------------------------------------

#[test]
fn spawn_gives_land_population_and_soldiers() {
    let game = start_two_players();
    let p1 = game.player(1).unwrap();
    assert!(p1.tiles > 20);
    assert!(p1.workers > 0 && p1.army() > 0);
    let owned = game.owners().iter().filter(|&&o| o == 1).count() as u32;
    assert_eq!(owned, p1.tiles, "compteur de cases incohérent");
}

#[test]
fn cannot_spawn_on_water_or_twice() {
    let setup = GameSetup {
        seed: 1,
        players: vec![PlayerSetup { name: "a".into() }, PlayerSetup { name: "b".into() }],
        config: config(),
    };
    let mut game = Game::new(&setup, test_map(20, 20, None)).unwrap();
    game.tick(&[
        (1, Intent::Spawn { tile: 0 }),
        (2, Intent::Spawn { tile: 21 + 5 }),
        (2, Intent::Spawn { tile: 21 + 9 * 20 }),
    ]);
    assert!(rejected(&game, 1, RejectReason::InvalidTile));
    assert!(rejected(&game, 2, RejectReason::AlreadySpawned));
}

// --- Population ------------------------------------------------------------

#[test]
fn population_grows_with_food() {
    let mut game = start_two_players();
    let before = game.population(1);
    run(&mut game, 300);
    assert!(game.population(1) > before, "la population doit croître");
    assert!(game.population(1) <= game.housing(1));
}

#[test]
fn famine_shrinks_population() {
    let mut game = start_two_players();
    // Que des soldats, aucun cueilleur, plus de nourriture.
    let p = game.player_mut(1);
    p.plan.soldiers_permille = 1000;
    p.soldiers = [p.workers + p.army(), 0, 0];
    p.workers = 0;
    p.resources = Cost::default();
    let before = game.population(1);
    run(&mut game, 50);
    assert!(game.population(1) < before, "sans nourriture, la population baisse");
}

#[test]
fn plan_trains_soldiers_over_time_and_costs_resources() {
    let mut game = start_two_players();
    let army = soldiers(&game, 1);
    game.tick(&[(1, Intent::SetPlan {
        plan: PopulationPlan { soldiers_permille: 600, ..Default::default() },
    })]);
    let after_one = soldiers(&game, 1);
    assert!(after_one - army < game.population(1) / 10, "la formation est progressive");
    run(&mut game, 200);
    assert!(soldiers(&game, 1) > army, "la formation doit avancer");

    // Sans ressources, personne n'est formé.
    let mut poor = start_two_players();
    poor.player_mut(1).resources = Cost::default();
    poor.tick(&[(1, Intent::SetPlan {
        plan: PopulationPlan { soldiers_permille: 600, ..Default::default() },
    })]);
    assert_eq!(soldiers(&poor, 1), army, "former un soldat coûte des ressources");
}

#[test]
fn garrison_trains_and_defends() {
    let mut game = start_two_players();
    rich(&mut game, 1);
    game.tick(&[(1, Intent::SetPlan {
        plan: PopulationPlan { soldiers_permille: 0, garrison_permille: 500, ..Default::default() },
    })]);
    run(&mut game, 400);
    let early = game.player(1).unwrap().garrison_training;
    let defense_early = game.defense_power(1);
    run(&mut game, 1500);
    let p = game.player(1).unwrap();
    assert!(p.garrison > 0);
    assert!(p.garrison_training > early, "la garnison s'entraîne : {early} → {} (pop {} / {})", p.garrison_training, game.population(1), game.housing(1));
    assert!(game.defense_power(1) > defense_early, "{} {} | {:?} g{} t{} e{}", game.defense_power(1), defense_early, p.soldiers, p.garrison, p.garrison_training, p.equipment);
}

#[test]
fn workers_follow_jobs() {
    let mut game = start_two_players();
    run(&mut game, 1);
    let p = game.player(1).unwrap();
    assert!(p.jobs[Job::Gatherer.index()] > 0, "il faut des cueilleurs");
    assert_eq!(p.jobs[Job::Miner.index()], 0, "pas de mineurs à l'âge du Bois");
    assert_eq!(p.jobs.iter().sum::<i64>(), p.workers);
}

// --- Âges et bâtiments --------------------------------------------------------

/// Construit un bâtiment et attend la fin du chantier.
fn build_done(game: &mut Game, id: PlayerId, tile: TileIdx, kind: BuildingKind) {
    game.tick(&[(id, Intent::Build { tile, kind })]);
    assert!(
        game.building_at(tile).is_some_and(|b| b.kind == kind),
        "chantier refusé : {:?}",
        game.last_update().events
    );
    let wait = game.config().building(kind).build_ticks + 1;
    run(game, wait);
}

#[test]
fn quarry_unlocks_miners() {
    let mut map = test_map(60, 30, None);
    let quarry = map.idx(12, 15) as usize;
    map.deposit[quarry] = Deposit::Quarry as u8;
    let mut game = start(map, &[(10, 15), (50, 15)]);
    rich(&mut game, 1);
    assert!(!game.job_unlocked(1, Job::Miner), "pas de mineurs sans carrière");
    build_done(&mut game, 1, quarry as TileIdx, BuildingKind::Quarry);
    assert!(game.job_unlocked(1, Job::Miner), "la carrière débloque les mineurs");
    assert!(game.job_slots(1).total(Job::Miner) > 0);
}

#[test]
fn archery_unlocks_archers() {
    let mut game = start_two_players();
    rich(&mut game, 1);
    assert!(!game.unit_unlocked(1, Unit::Ranged));
    let tile = game.map().idx(10, 15);
    build_done(&mut game, 1, tile, BuildingKind::Archery);
    assert!(game.unit_unlocked(1, Unit::Ranged));
}

#[test]
fn buildings_need_their_deposit() {
    let mut map = test_map(60, 30, None);
    let grove = map.idx(12, 15) as usize;
    map.deposit[grove] = Deposit::Grove as u8;
    let mut game = start(map, &[(10, 15), (50, 15)]);
    rich(&mut game, 1);

    let plain = game.map().idx(9, 15);
    game.tick(&[(1, Intent::Build { tile: plain, kind: BuildingKind::Sawmill })]);
    assert!(rejected(&game, 1, RejectReason::NeedsDeposit));

    let jobs_before = game.job_slots(1).total(Job::Lumberjack);
    build_done(&mut game, 1, grove as TileIdx, BuildingKind::Sawmill);
    assert!(game.job_slots(1).total(Job::Lumberjack) > jobs_before, "la scierie crée des emplois");
}

// --- Académie ----------------------------------------------------------------

#[test]
fn research_needs_academy_and_one_at_a_time() {
    let mut game = start_two_players();
    rich(&mut game, 1);
    game.tick(&[(1, Intent::Research { tech: TechId::Agriculture })]);
    assert!(rejected(&game, 1, RejectReason::NoAcademy));

    let tile = game.map().idx(10, 15);
    build_done(&mut game, 1, tile, BuildingKind::Academy);
    let other = game.map().idx(12, 12);
    game.tick(&[(1, Intent::Build { tile: other, kind: BuildingKind::Academy })]);
    assert!(rejected(&game, 1, RejectReason::MaxCount), "une seule académie");

    game.tick(&[(1, Intent::Research { tech: TechId::Agriculture })]);
    assert!(game.player(1).unwrap().research.is_some());
    game.tick(&[(1, Intent::Research { tech: TechId::Forestry })]);
    assert!(rejected(&game, 1, RejectReason::Researching));
}

#[test]
fn research_raises_its_bonus_and_costs_more_each_level() {
    let mut game = start_two_players();
    rich(&mut game, 1);
    let tile = game.map().idx(10, 15);
    build_done(&mut game, 1, tile, BuildingKind::Academy);

    let effect = Effect::JobYield(Job::Gatherer);
    assert_eq!(game.modifier(1, effect), 1000);
    let (first, ticks) = game.tech_price(1, TechId::Agriculture).unwrap();
    game.tick(&[(1, Intent::Research { tech: TechId::Agriculture })]);
    run(&mut game, ticks + 1);
    assert_eq!(game.tech_level(1, TechId::Agriculture), 1);
    assert!(game.modifier(1, effect) > 1000, "le bonus s'applique");
    let (second, _) = game.tech_price(1, TechId::Agriculture).unwrap();
    assert!(second.food > first.food && second.stone > 0, "le niveau 2 coûte plus, et de la pierre");
}

#[test]
fn tech_prerequisites_are_enforced() {
    let mut game = start_two_players();
    rich(&mut game, 1);
    let tile = game.map().idx(10, 15);
    build_done(&mut game, 1, tile, BuildingKind::Academy);
    // L'urbanisme demande l'architecture ; la métallurgie demande une mine.
    assert_eq!(game.can_research(1, TechId::Urbanism), Err(RejectReason::Locked));
    assert_eq!(game.can_research(1, TechId::Metallurgy), Err(RejectReason::Locked));
    assert!(game.can_research(1, TechId::Architecture).is_ok());
}

#[test]
fn blades_make_warriors_hit_harder() {
    let mut game = start_two_players();
    let units = [100_000, 0, 0];
    let before = game.attack_power(1, &units);
    game.player_mut(1).techs[TechId::Blades.index()] = 2;
    assert!(game.attack_power(1, &units) > before);
    assert_eq!(game.attack_power(2, &units), before, "seulement pour celui qui a la technologie");
}

#[test]
fn building_costs_grow() {
    let mut game = start_two_players();
    rich(&mut game, 1);
    let first = game.build_cost(1, BuildingKind::Barracks);
    let tile = game.map().idx(10, 15);
    game.tick(&[(1, Intent::Build { tile, kind: BuildingKind::Barracks })]);
    assert!(game.build_cost(1, BuildingKind::Barracks).wood > first.wood);
}

#[test]
fn captured_fort_is_destroyed() {
    let mut game = start(test_map(40, 20, None), &[(10, 10), (30, 10)]);
    rich(&mut game, 2);
    let fort = game.map().idx(30, 10);
    game.tick(&[(2, Intent::Build { tile: fort, kind: BuildingKind::Fort })]);
    run(&mut game, 400);
    assert_eq!(game.player(2).unwrap().buildings[BuildingKind::Fort.index()], 1);
    game.set_owner(fort, 1);
    assert_eq!(game.building_at(fort).unwrap().owner, NEUTRAL, "il reste une ruine");
    assert_eq!(game.player(1).unwrap().buildings[BuildingKind::Fort.index()], 0);
    assert_eq!(game.player(2).unwrap().buildings[BuildingKind::Fort.index()], 0);
}

#[test]
fn captured_building_changes_owner() {
    let mut game = start(test_map(40, 20, None), &[(10, 10), (30, 10)]);
    rich(&mut game, 2);
    let city = game.map().idx(30, 10);
    game.tick(&[(2, Intent::Build { tile: city, kind: BuildingKind::City })]);
    run(&mut game, 400);
    assert_eq!(game.player(2).unwrap().buildings[BuildingKind::City.index()], 1);
    game.set_owner(city, 1);
    assert_eq!(game.building_at(city).unwrap().owner, 1);
    assert_eq!(game.player(1).unwrap().buildings[BuildingKind::City.index()], 1);
    assert_eq!(game.player(2).unwrap().buildings[BuildingKind::City.index()], 0);
}

// --- Combat ------------------------------------------------------------------

#[test]
fn attacking_neutral_land_expands_territory() {
    let mut game = start_two_players();
    let before = game.player(1).unwrap().tiles;
    game.tick(&[(1, Intent::Attack { target: NEUTRAL, ratio_permille: 800 })]);
    run(&mut game, 100);
    assert!(game.player(1).unwrap().tiles > before);
}

#[test]
fn attack_without_border_is_rejected() {
    let mut game = start_two_players();
    game.tick(&[(1, Intent::Attack { target: 2, ratio_permille: 500 })]);
    assert!(rejected(&game, 1, RejectReason::NoBorder));
}

#[test]
fn immunity_blocks_early_attacks() {
    let mut game = start_with(test_map(60, 30, None), &[(10, 15), (50, 15)], GameConfig::default());
    split_map(&mut game);
    game.tick(&[(1, Intent::Attack { target: 2, ratio_permille: 500 })]);
    assert!(rejected(&game, 1, RejectReason::Immune));
}

/// Prend des cases à un défenseur de force donnée ; renvoie (cases prises, pertes).
fn duel(defender_soldiers: i64, defender_garrison: i64) -> (u32, i64) {
    let mut game = start_two_players();
    split_map(&mut game);
    game.player_mut(1).soldiers = [500_000, 0, 0];
    game.player_mut(2).soldiers = [defender_soldiers * 1000, 0, 0];
    game.player_mut(2).garrison = defender_garrison * 1000;
    // Personne ne forme ni ne renvoie de soldats pendant le duel.
    for id in [1, 2] {
        game.player_mut(id).plan.soldiers_permille = 1000;
        game.player_mut(id).resources = Cost::default();
    }
    let tiles = game.player(1).unwrap().tiles;
    game.tick(&[(1, Intent::Attack { target: 2, ratio_permille: 1000 })]);
    run(&mut game, 30);
    let taken = game.player(1).unwrap().tiles - tiles;
    let left = game.away_soldiers(1).iter().sum::<i64>() + soldiers(&game, 1);
    (taken, 500_000 - left)
}

#[test]
fn weak_defender_falls_fast_strong_defender_resists() {
    let (fast, cheap) = duel(10, 0);
    let (slow, costly) = duel(800, 0);
    assert!(fast > slow * 2, "défenseur faible : {fast} cases, fort : {slow}");
    assert!(cheap < costly, "pertes face au faible {cheap}, face au fort {costly}");
}

#[test]
fn trained_garrison_is_hard_to_break() {
    let (open, _) = duel(0, 0);
    let (guarded, _) = duel(0, 600);
    assert!(guarded < open, "garnison : {guarded} cases contre {open}");
}

#[test]
fn retreat_costs_troops() {
    let mut game = start_two_players();
    split_map(&mut game);
    game.player_mut(1).plan.soldiers_permille = 1000;
    game.player_mut(1).resources = Cost::default();
    game.tick(&[(1, Intent::Attack { target: 2, ratio_permille: 1000 })]);
    let sent = game.away_soldiers(1).iter().sum::<i64>();
    let id = game.attacks()[0].id;
    game.tick(&[(1, Intent::CancelAttack { attack_id: id })]);
    assert!(soldiers(&game, 1) < sent, "une retraite coûte des soldats");
}

#[test]
fn same_inputs_same_hash() {
    let play = || {
        let mut game = start_two_players();
        for t in 0..400 {
            let intents = match t % 50 {
                0 => vec![
                    (1, Intent::Attack { target: NEUTRAL, ratio_permille: 600 }),
                    (2, Intent::Attack { target: NEUTRAL, ratio_permille: 700 }),
                ],
                25 => vec![(1, Intent::Attack { target: 2, ratio_permille: 600 })],
                _ => vec![],
            };
            game.tick(&intents);
        }
        game.state_hash()
    };
    assert_eq!(play(), play());
}

// --- Bateaux --------------------------------------------------------------------

#[test]
fn boat_crosses_to_another_island() {
    let mut game = start(test_map(60, 30, Some((28, 32))), &[(10, 15), (50, 15)]);
    game.player_mut(1).soldiers = [400_000, 0, 0];
    game.player_mut(1).plan.soldiers_permille = 1000;
    game.tick(&[(1, Intent::Attack { target: NEUTRAL, ratio_permille: 1000 })]);
    run(&mut game, 300);
    game.player_mut(1).soldiers = [200_000, 0, 0];
    rich(&mut game, 1);
    let landing = game.map().idx(33, 5);
    game.tick(&[(1, Intent::Naval { tile: landing, ratio_permille: 800 })]);
    assert_eq!(game.boats().len(), 1, "le bateau doit partir : {:?}", game.last_update().events);
    run(&mut game, 100);
    assert!(game.boats().is_empty());
    let east = (0..game.map().len() as TileIdx)
        .filter(|&t| game.map().xy(t).0 > 32 && game.owner(t) == 1)
        .count();
    assert!(east > 1, "le joueur 1 doit avoir pris pied sur l'autre île");
}

// --- Rythme des attaques ---------------------------------------------------

/// Durée (s) et cases prises par une expédition de `n` soldats sur des terres
/// libres d'une grande prairie.
fn neutral_expedition(n: i64) -> (f64, u32) {
    let mut game = start(test_map(400, 200, None), &[(200, 100)]);
    game.player_mut(1).soldiers = [n * 1000, 0, 0];
    game.player_mut(1).plan.soldiers_permille = 1000;
    game.player_mut(1).resources = Cost::default();
    let tiles = game.player(1).unwrap().tiles;
    game.tick(&[(1, Intent::Attack { target: NEUTRAL, ratio_permille: 1000 })]);
    let mut ticks = 0;
    while !game.attacks().is_empty() && ticks < 20_000 {
        game.tick(&[]);
        ticks += 1;
    }
    (ticks as f64 / 10.0, game.player(1).unwrap().tiles - tiles)
}

/// Colonnes gagnées par seconde quand `attack` soldats attaquent un voisin
/// qui en a `defense` (front de 198 cases).
fn front_speed(attack: i64, defense: i64) -> f64 {
    let mut game = start(test_map(400, 200, None), &[(100, 100), (300, 100)]);
    split_map(&mut game);
    game.player_mut(1).soldiers = [attack * 1000, 0, 0];
    game.player_mut(2).soldiers = [defense * 1000, 0, 0];
    for id in [1, 2] {
        game.player_mut(id).plan.soldiers_permille = 1000;
        game.player_mut(id).resources = Cost::default();
    }
    let tiles = game.player(1).unwrap().tiles;
    game.tick(&[(1, Intent::Attack { target: 2, ratio_permille: 1000 })]);
    run(&mut game, 50);
    (game.player(1).unwrap().tiles - tiles) as f64 / 198.0 / 5.0
}

#[test]
#[ignore]
fn measure_attack_pace() {
    for n in [300, 1000, 3000] {
        let (s, t) = neutral_expedition(n);
        println!("terres libres, {n} soldats : {s:.1} s, {t} cases");
    }
    for (a, d) in [(5000, 2500), (5000, 5000), (2500, 5000)] {
        println!("{a} contre {d} : {:.2} colonnes/s", front_speed(a, d));
    }
}

#[test]
fn expeditions_end_and_whole_front_advances() {
    // Une expédition sur des terres libres s'épuise en moins d'une minute.
    let (seconds, tiles) = neutral_expedition(3000);
    assert!(seconds < 60.0 && tiles > 1000, "{seconds} s, {tiles} cases");
    // Le front avance d'au moins une colonne par seconde à forces égales,
    // et plus vite face à un défenseur faible.
    let even = front_speed(5000, 5000);
    assert!(even >= 1.0, "{even} colonnes/s");
    assert!(front_speed(5000, 2500) > even * 1.5);
}

#[test]
#[ignore]
fn measure_expansion_shape() {
    let mut game = start(test_map(200, 200, None), &[(100, 100)]);
    let stats = |g: &Game| {
        let (mut sx, mut sy, mut n) = (0i64, 0i64, 0i64);
        let (mut x0, mut x1, mut y0, mut y1) = (u32::MAX, 0, u32::MAX, 0);
        for t in 0..g.map().len() as TileIdx {
            if g.owner(t) == 1 {
                let (x, y) = g.map().xy(t);
                sx += x as i64; sy += y as i64; n += 1;
                x0 = x0.min(x); x1 = x1.max(x); y0 = y0.min(y); y1 = y1.max(y);
            }
        }
        format!("{n} cases, centre ({:.1}, {:.1}), {} × {}", sx as f64 / n as f64, sy as f64 / n as f64, x1 - x0 + 1, y1 - y0 + 1)
    };
    println!("départ : {}", stats(&game));
    for i in 0..8 {
        game.player_mut(1).soldiers = [60_000, 0, 0];
        game.tick(&[(1, Intent::Attack { target: NEUTRAL, ratio_permille: 500 })]);
        run(&mut game, 100);
        println!("expédition {} : {}", i + 1, stats(&game));
    }
}

#[test]
fn short_expeditions_grow_evenly() {
    // Des expéditions courtes ne doivent pas toutes partir du même côté.
    let mut game = start(test_map(200, 200, None), &[(100, 100)]);
    for _ in 0..8 {
        game.player_mut(1).soldiers = [60_000, 0, 0];
        game.tick(&[(1, Intent::Attack { target: NEUTRAL, ratio_permille: 500 })]);
        run(&mut game, 300);
    }
    let (mut sx, mut sy, mut n) = (0i64, 0i64, 0i64);
    for t in 0..game.map().len() as TileIdx {
        if game.owner(t) == 1 {
            let (x, y) = game.map().xy(t);
            sx += x as i64;
            sy += y as i64;
            n += 1;
        }
    }
    let (cx, cy) = (sx as f64 / n as f64, sy as f64 / n as f64);
    assert!((cx - 100.0).abs() < 2.0 && (cy - 100.0).abs() < 2.0, "centre ({cx:.1}, {cy:.1})");
}

// --- Aperçus pour l'interface --------------------------------------------

/// Cases réellement prises par une attaque menée jusqu'au bout.
fn real_attack(game: &mut Game, target: PlayerId, ratio: u16) -> u32 {
    let before = game.player(1).unwrap().tiles;
    game.tick(&[(1, Intent::Attack { target, ratio_permille: ratio })]);
    let mut n = 0;
    while !game.attacks().is_empty() && n < 20_000 {
        game.tick(&[]);
        n += 1;
    }
    game.player(1).unwrap().tiles - before
}

fn frozen(game: &mut Game) {
    for id in 1..=game.players().len() as PlayerId {
        game.player_mut(id).plan.soldiers_permille = 1000;
        game.player_mut(id).resources = Cost::default();
    }
}

#[test]
fn attack_estimate_matches_the_real_fight() {
    for (attack, defense) in [(3000, 600), (3000, 3000), (1500, 4000)] {
        let mut game = start(test_map(300, 150, None), &[(75, 75), (225, 75)]);
        split_map(&mut game);
        game.player_mut(1).soldiers = [attack * 1000, 0, 0];
        game.player_mut(2).soldiers = [defense * 1000, 0, 0];
        frozen(&mut game);
        let estimate = game.estimate_attack(1, 2, 1000).unwrap();
        let real = real_attack(&mut game, 2, 1000);
        let (e, r) = (estimate.tiles as f64, real.max(1) as f64);
        assert!(e / r > 0.6 && e / r < 1.6, "{attack} contre {defense} : estimé {e}, réel {r}");
    }
}

#[test]
fn neutral_estimate_matches_the_real_expedition() {
    let mut game = start(test_map(300, 300, None), &[(150, 150)]);
    game.player_mut(1).soldiers = [400_000, 0, 0];
    frozen(&mut game);
    let estimate = game.estimate_attack(1, NEUTRAL, 500).unwrap().tiles as f64;
    let real = real_attack(&mut game, NEUTRAL, 500) as f64;
    assert!(estimate / real > 0.8 && estimate / real < 1.25, "estimé {estimate}, réel {real}");
}

#[test]
fn plan_outcome_predicts_fewer_workers_and_more_defense() {
    let mut game = start_two_players();
    run(&mut game, 50);
    let mut plan = game.player(1).unwrap().plan.clone();
    plan.soldiers_permille = 100;
    let few = game.plan_outcome(1, &plan);
    plan.soldiers_permille = 600;
    let many = game.plan_outcome(1, &plan);
    assert!(many.workers < few.workers);
    assert!(many.defense > few.defense);
    assert!(many.production.food + many.production.wood < few.production.food + few.production.wood);
}

// --- Expérience, équipement, intégration ------------------------------------

#[test]
fn workers_gain_experience_and_produce_more() {
    let mut game = start_two_players();
    run(&mut game, 20);
    let fresh = game.production(1).food + game.production(1).wood;
    run(&mut game, 3000);
    let share = game.experience_share(1);
    assert!(share[0] > 500, "expérience des cueilleurs : {share:?}");
    // Même répartition, même nombre de travailleurs : seule l'expérience change.
    let jobs = game.player(1).unwrap().jobs;
    let mut novices = game.player(1).unwrap().clone();
    novices.veterans = [0; 4];
    let veteran_output = game.production_for(1, &jobs);
    *game.player_mut(1) = novices;
    let novice_output = game.production_for(1, &jobs);
    assert!(veteran_output.food > novice_output.food * 11 / 10, "{veteran_output:?} contre {novice_output:?}");
    let _ = fresh;
}

#[test]
fn moving_workers_away_loses_their_experience() {
    let mut game = start_two_players();
    run(&mut game, 3000);
    let before = game.player(1).unwrap().veterans[0];
    assert!(before > 0);
    // Tous les cueilleurs passent bûcherons : leur expérience de cueilleur est perdue.
    let plan = PopulationPlan { job_mix: Some([0, 10, 0, 0]), ..game.player(1).unwrap().plan.clone() };
    game.tick(&[(1, Intent::SetPlan { plan })]);
    run(&mut game, 5);
    assert_eq!(game.player(1).unwrap().veterans[0], 0);
}

#[test]
fn iron_equips_recruits_and_makes_them_stronger() {
    let mut game = start_two_players();
    game.player_mut(1).resources = Cost::new(100_000, 100_000, 0, 100_000).to_milli();
    game.player_mut(2).resources = Cost::new(100_000, 100_000, 0, 0).to_milli();
    let plan = PopulationPlan { soldiers_permille: 600, ..Default::default() };
    game.tick(&[(1, Intent::SetPlan { plan: plan.clone() }), (2, Intent::SetPlan { plan })]);
    run(&mut game, 600);
    assert!(game.player(1).unwrap().equipment > 300, "équipement {}", game.player(1).unwrap().equipment);
    assert_eq!(game.player(2).unwrap().equipment, 0);
    let units = [100_000, 0, 0];
    assert!(game.attack_power(1, &units) > game.attack_power(2, &units) * 11 / 10);
}

#[test]
fn conquered_land_must_be_integrated() {
    let mut game = start_two_players();
    split_map(&mut game);
    let housing = game.housing(1);
    // Le joueur 1 prend une bande au joueur 2.
    for tile in 0..game.map().len() as TileIdx {
        let (x, _) = game.map().xy(tile);
        if game.owner(tile) == 2 && x < 40 {
            game.set_owner(tile, 1);
        }
    }
    let taken = game.unsettled_tiles(1);
    assert!(taken > 0);
    assert_eq!(game.housing(1), housing, "les conquêtes ne logent personne au début");
    let wait = game.config().integration_ticks + 2;
    run(&mut game, wait);
    assert_eq!(game.unsettled_tiles(1), 0);
    assert!(game.housing(1) > housing);
}
