//! Session de jeu : relie la configuration du salon (qui joue, avec quelle
//! IA, sur quelle carte) au moteur.
//!
//! Le même code sert au jeu solo dans le navigateur (via `frontier-wasm`),
//! au futur serveur multijoueur et au futur module Python d'entraînement.

use frontier_ai::{create_agent, BotHost, Difficulty};
use frontier_engine::rng::hash2;
use frontier_engine::{
    BuildingKind, Cost, Game, GameConfig, GameEvent, GameSetup, Intent, Job, Phase, PlayerId, PlayerSetup,
    PopulationPlan, Research, SetupError, TechId, TickUpdate, TileIdx, Unit, MILLI,
};
use frontier_worldgen::{generate, MapSize, WorldParams};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Controller {
    Human,
    Bot { agent: String, difficulty: Difficulty },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Seat {
    pub name: String,
    pub controller: Controller,
}

/// Configuration d'un salon, telle qu'envoyée par le menu.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionSetup {
    pub seed: u64,
    pub map_size: MapSize,
    pub seats: Vec<Seat>,
    #[serde(default)]
    pub config: GameConfig,
}

#[derive(Debug)]
pub enum SessionError {
    Setup(SetupError),
    UnknownAgent(String),
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::Setup(e) => write!(f, "{e}"),
            SessionError::UnknownAgent(id) => write!(f, "IA inconnue : {id}"),
        }
    }
}

impl std::error::Error for SessionError {}

pub struct Session {
    game: Game,
    bots: BotHost,
    queue: Vec<(PlayerId, Intent)>,
}

impl Session {
    pub fn new(setup: &SessionSetup) -> Result<Session, SessionError> {
        let map = generate(&WorldParams::new(setup.seed, setup.map_size));
        let game_setup = GameSetup {
            seed: hash2(setup.seed, 0x5E55),
            players: setup
                .seats
                .iter()
                .map(|seat| PlayerSetup {
                    name: seat.name.clone(),
                })
                .collect(),
            config: setup.config.clone(),
        };
        let game = Game::new(&game_setup, map).map_err(SessionError::Setup)?;

        let mut bots = BotHost::new();
        for (i, seat) in setup.seats.iter().enumerate() {
            if let Controller::Bot { agent, difficulty } = &seat.controller {
                let player = i as PlayerId + 1;
                let brain = create_agent(agent, *difficulty, hash2(setup.seed, player as u64))
                    .ok_or_else(|| SessionError::UnknownAgent(agent.clone()))?;
                bots.add(player, brain);
            }
        }

        Ok(Session {
            game,
            bots,
            queue: Vec::new(),
        })
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    /// Ordre d'un joueur humain (ou distant), appliqué au prochain tour.
    pub fn submit(&mut self, player: PlayerId, intent: Intent) {
        self.queue.push((player, intent));
    }

    /// Calcule un tour : les IA réfléchissent, puis le moteur avance.
    pub fn step(&mut self) -> &TickUpdate {
        self.bots.collect(&self.game, &mut self.queue);
        let intents = std::mem::take(&mut self.queue);
        self.game.tick(&intents)
    }

    /// Résumé de l'état pour l'interface (personnes et ressources en unités entières).
    pub fn snapshot(&self) -> Snapshot<'_> {
        let game = &self.game;
        let per_second = 1000 / game.config().tick_ms.max(1) as i64;
        Snapshot {
            tick: game.tick_count(),
            phase: game.phase(),
            land_tiles: game.land_tiles(),
            immunity: game.in_immunity(),
            players: game.players().iter().map(|p| player_view(game, p.id, per_second)).collect(),
            attacks: game
                .attacks()
                .iter()
                .map(|a| AttackView {
                    id: a.id,
                    attacker: a.attacker,
                    target: a.target,
                    troops: a.soldiers() / MILLI,
                })
                .collect(),
            boats: game
                .boats()
                .iter()
                .map(|b| BoatView {
                    id: b.id,
                    owner: b.owner,
                    tile: b.tile(),
                    destination: b.destination(),
                    troops: b.soldiers() / MILLI,
                })
                .collect(),
            buildings: game
                .buildings()
                .iter()
                .map(|b| BuildingView {
                    id: b.id,
                    kind: b.kind,
                    tile: b.tile,
                    owner: b.owner,
                    remaining: b.remaining,
                })
                .collect(),
            events: &game.last_update().events,
        }
    }
}

fn units(cost: Cost) -> Cost {
    Cost::new(cost.food / MILLI, cost.wood / MILLI, cost.stone / MILLI, cost.iron / MILLI)
}

fn player_view<'a>(game: &'a Game, id: PlayerId, per_second: i64) -> PlayerView<'a> {
    let p = game.player(id).expect("joueur");
    let slots = game.job_slots(id);
    let production = game.production(id);
    PlayerView {
        id,
        name: &p.name,
        alive: p.alive,
        spawned: p.spawned,
        tiles: p.tiles,
        techs: p.techs,
        research: p.research,
        tech_costs: TechId::ALL.map(|t| game.tech_price(id, t).map(|(c, _)| units(c))),
        tech_ticks: TechId::ALL.map(|t| game.tech_price(id, t).map_or(0, |(_, ticks)| ticks)),
        tech_locked: TechId::ALL.map(|t| !game.tech_unlocked(id, t)),
        population: game.population(id) / MILLI,
        housing: game.housing(id) / MILLI,
        workers: p.workers / MILLI,
        jobs: p.jobs.map(|j| j / MILLI),
        job_slots: Job::ALL.map(|j| slots.total(j) / MILLI),
        soldiers: p.soldiers.map(|s| s / MILLI),
        away: game.troops_away(id),
        garrison: p.garrison / MILLI,
        garrison_training: p.garrison_training,
        defense: game.defense_power(id) / MILLI,
        resources: units(p.resources),
        storage: game.storage(id) / MILLI,
        production: units(production.scaled(per_second, 1)),
        food_consumption: game.food_consumption(id) * per_second / MILLI,
        training_spend: units(p.training_spend.scaled(per_second, 1)),
        training_rate: game.training_rate(id) * per_second / MILLI,
        plan: &p.plan,
        unit_unlocked: Unit::ALL.map(|u| game.unit_unlocked(id, u)),
        buildings: p.buildings,
        build_costs: BuildingKind::ALL.map(|k| units(game.build_cost(id, k))),
    }
}

#[derive(Serialize)]
pub struct Snapshot<'a> {
    pub tick: u64,
    pub phase: &'a Phase,
    pub land_tiles: u32,
    pub immunity: bool,
    pub players: Vec<PlayerView<'a>>,
    pub attacks: Vec<AttackView>,
    pub boats: Vec<BoatView>,
    pub buildings: Vec<BuildingView>,
    pub events: &'a [GameEvent],
}

/// Vue d'un joueur pour l'interface. Personnes et ressources en unités,
/// production et consommation par seconde.
#[derive(Serialize)]
pub struct PlayerView<'a> {
    pub id: PlayerId,
    pub name: &'a str,
    pub alive: bool,
    pub spawned: bool,
    pub tiles: u32,
    /// Niveau de chaque technologie, recherche en cours.
    pub techs: [u8; 17],
    pub research: Option<Research>,
    /// Coût (unités) et durée (tours) du prochain niveau ; `None` au maximum.
    pub tech_costs: [Option<Cost>; 17],
    pub tech_ticks: [u32; 17],
    /// Prérequis manquant (technologie ou bâtiment).
    pub tech_locked: [bool; 17],
    pub population: i64,
    pub housing: i64,
    pub workers: i64,
    pub jobs: [i64; 4],
    pub job_slots: [i64; 4],
    pub soldiers: [i64; 3],
    pub away: i64,
    pub garrison: i64,
    pub garrison_training: i64,
    pub defense: i64,
    pub resources: Cost,
    pub storage: i64,
    pub production: Cost,
    /// Dépense par seconde pour former soldats et garnison.
    pub training_spend: Cost,
    pub food_consumption: i64,
    pub training_rate: i64,
    pub plan: &'a PopulationPlan,
    pub unit_unlocked: [bool; 3],
    pub buildings: [u32; 10],
    pub build_costs: [Cost; 10],
}

#[derive(Serialize)]
pub struct AttackView {
    pub id: u32,
    pub attacker: PlayerId,
    pub target: PlayerId,
    pub troops: i64,
}

#[derive(Serialize)]
pub struct BoatView {
    pub id: u32,
    pub owner: PlayerId,
    pub tile: TileIdx,
    pub destination: TileIdx,
    pub troops: i64,
}

#[derive(Serialize)]
pub struct BuildingView {
    pub id: u32,
    pub kind: BuildingKind,
    pub tile: TileIdx,
    pub owner: PlayerId,
    pub remaining: u32,
}
