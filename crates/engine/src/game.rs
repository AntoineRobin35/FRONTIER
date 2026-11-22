//! État de la partie et ordonnancement d'un tour.
//!
//! Chaque système vit dans son propre module (`spawn`, `population`, `attack`,
//! `naval`, `buildings`) sous forme de méthodes de `Game`. Ce fichier ne
//! contient que l'état, la lecture de l'état et l'ordre d'exécution.
//!
//! Un tour (`Game::tick`) se déroule toujours dans cet ordre :
//! 1. application des ordres reçus, dans l'ordre donné ;
//! 2. phase de placement : début de partie quand tout le monde est placé ;
//! 3. population : production, nourriture, naissances, formation, recherche ;
//! 4. chantiers ;
//! 5. avancée des attaques terrestres ;
//! 6. déplacement et débarquement des bateaux ;
//! 7. éliminations, puis victoire.

use serde::{Deserialize, Serialize};

use crate::attack::Attack;
use crate::buildings::Building;
use crate::config::{Cost, GameConfig};
use crate::intent::Intent;
use crate::map::{MapData, MapError, Regions, TileIdx};
use crate::naval::Boat;
use crate::player::{Player, PlayerId, NEUTRAL};
use crate::rng::Rng;
use crate::terrain::Terrain;
use crate::update::{GameEvent, RejectReason, TickUpdate};

pub const MAX_PLAYERS: usize = 64;

/// Valeur de `building_at` pour une case sans bâtiment.
pub(crate) const NO_BUILDING: u32 = u32::MAX;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Phase {
    /// Les joueurs choisissent leur point de départ.
    Spawn,
    Playing,
    Ended { winner: Option<PlayerId> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerSetup {
    pub name: String,
}

/// Tout ce qu'il faut (avec la carte) pour créer une partie identique sur
/// toutes les machines.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameSetup {
    /// Graine de l'aléatoire de la partie (distincte de celle de la carte).
    pub seed: u64,
    pub players: Vec<PlayerSetup>,
    #[serde(default)]
    pub config: GameConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupError {
    NoPlayers,
    TooManyPlayers,
    InvalidMap(MapError),
    NoLand,
}

impl std::fmt::Display for SetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetupError::NoPlayers => write!(f, "aucun joueur"),
            SetupError::TooManyPlayers => write!(f, "trop de joueurs (max {MAX_PLAYERS})"),
            SetupError::InvalidMap(e) => write!(f, "carte invalide : {e}"),
            SetupError::NoLand => write!(f, "la carte ne contient aucune terre"),
        }
    }
}

impl std::error::Error for SetupError {}

pub struct Game {
    pub(crate) config: GameConfig,
    pub(crate) map: MapData,
    pub(crate) regions: Regions,
    pub(crate) land_tiles: u32,
    pub(crate) owner: Vec<PlayerId>,
    pub(crate) players: Vec<Player>,
    /// Nombre de cases de chaque terrain possédées par chaque joueur
    /// (indexé par `id - 1`) : la production se calcule sans parcourir la carte.
    pub(crate) terrain_tiles: Vec<[u32; Terrain::COUNT]>,
    /// Dernier joueur à avoir pris une case à chaque joueur (pour le journal).
    pub(crate) last_conqueror: Vec<Option<PlayerId>>,
    pub(crate) attacks: Vec<Attack>,
    pub(crate) boats: Vec<Boat>,
    pub(crate) buildings: Vec<Building>,
    /// Index dans `buildings` du bâtiment de chaque case, ou `NO_BUILDING`.
    pub(crate) building_at: Vec<u32>,
    pub(crate) next_id: u32,
    pub(crate) phase: Phase,
    /// Tour du début de la partie (fin du placement).
    pub(crate) started_at: Option<u64>,
    pub(crate) tick: u64,
    pub(crate) rng: Rng,
    pub(crate) update: TickUpdate,
}

impl Game {
    pub fn new(setup: &GameSetup, map: MapData) -> Result<Game, SetupError> {
        map.validate().map_err(SetupError::InvalidMap)?;
        if setup.players.is_empty() {
            return Err(SetupError::NoPlayers);
        }
        if setup.players.len() > MAX_PLAYERS {
            return Err(SetupError::TooManyPlayers);
        }
        let land_tiles = map.passable_count();
        if land_tiles == 0 {
            return Err(SetupError::NoLand);
        }

        let players = setup
            .players
            .iter()
            .enumerate()
            .map(|(i, p)| Player::new(i as PlayerId + 1, p.name.clone()))
            .collect::<Vec<_>>();
        let n = map.len();

        Ok(Game {
            config: setup.config.clone(),
            regions: Regions::compute(&map),
            land_tiles,
            owner: vec![NEUTRAL; n],
            terrain_tiles: vec![[0; Terrain::COUNT]; players.len()],
            last_conqueror: vec![None; players.len()],
            players,
            attacks: Vec::new(),
            boats: Vec::new(),
            buildings: Vec::new(),
            building_at: vec![NO_BUILDING; n],
            next_id: 1,
            phase: Phase::Spawn,
            started_at: None,
            tick: 0,
            rng: Rng::new(setup.seed),
            update: TickUpdate::default(),
            map,
        })
    }

    // ------------------------------------------------------------------
    // Lecture de l'état
    // ------------------------------------------------------------------

    pub fn config(&self) -> &GameConfig {
        &self.config
    }

    pub fn map(&self) -> &MapData {
        &self.map
    }

    pub fn regions(&self) -> &Regions {
        &self.regions
    }

    /// Nombre de cases franchissables de la carte.
    pub fn land_tiles(&self) -> u32 {
        self.land_tiles
    }

    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    /// Nombre de tours déjà calculés.
    pub fn tick_count(&self) -> u64 {
        self.tick
    }

    #[inline]
    pub fn owner(&self, tile: TileIdx) -> PlayerId {
        self.owner[tile as usize]
    }

    pub fn owners(&self) -> &[PlayerId] {
        &self.owner
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn player(&self, id: PlayerId) -> Option<&Player> {
        if id == NEUTRAL {
            return None;
        }
        self.players.get(id as usize - 1)
    }

    pub fn attacks(&self) -> &[Attack] {
        &self.attacks
    }

    pub fn boats(&self) -> &[Boat] {
        &self.boats
    }

    pub fn buildings(&self) -> &[Building] {
        &self.buildings
    }

    /// Bâtiment présent sur une case.
    pub fn building_at(&self, tile: TileIdx) -> Option<&Building> {
        match self.building_at[tile as usize] {
            NO_BUILDING => None,
            index => self.buildings.get(index as usize),
        }
    }

    pub fn last_update(&self) -> &TickUpdate {
        &self.update
    }

    /// Soldats partis en campagne ou en mer (personnes entières).
    pub fn troops_away(&self, id: PlayerId) -> i64 {
        self.away_soldiers(id).iter().sum::<i64>() / crate::population::MILLI
    }

    /// Tour du début de la partie (fin de la phase de placement).
    pub fn started_at(&self) -> Option<u64> {
        self.started_at
    }

    /// Vrai pendant l'immunité de début de partie.
    pub fn in_immunity(&self) -> bool {
        self.immune()
    }

    /// Vrai si une case voisine appartient à `player`.
    #[inline]
    pub fn touches(&self, tile: TileIdx, player: PlayerId) -> bool {
        self.map.neighbors(tile).any(|n| self.owner[n as usize] == player)
    }

    /// Vrai si la case touche l'eau.
    #[inline]
    pub fn is_coastal(&self, tile: TileIdx) -> bool {
        self.map.neighbors(tile).any(|n| self.map.terrain(n).is_water())
    }

    /// Empreinte de l'état complet : deux machines qui ont la même
    /// empreinte au même tour sont synchronisées.
    pub fn state_hash(&self) -> u64 {
        let mut h = Fnv::new();
        h.write_u64(self.tick);
        for &o in &self.owner {
            h.write_u64(o as u64);
        }
        for p in &self.players {
            h.write_u64(p.tiles as u64);
            h.write_u64(p.alive as u64);
            for v in [p.resources.food, p.resources.wood, p.resources.stone, p.resources.iron] {
                h.write_u64(v as u64);
            }
            h.write_u64(p.workers as u64);
            h.write_u64(p.garrison as u64);
            h.write_u64(p.garrison_training as u64);
            for t in p.techs {
                h.write_u64(t as u64);
            }
            for s in p.soldiers {
                h.write_u64(s as u64);
            }
        }
        for a in &self.attacks {
            h.write_u64(a.id as u64);
            for u in a.units {
                h.write_u64(u as u64);
            }
        }
        for b in &self.boats {
            h.write_u64(b.id as u64);
            h.write_u64(b.soldiers() as u64);
            h.write_u64(b.progress as u64);
        }
        for b in &self.buildings {
            h.write_u64(b.id as u64);
            h.write_u64(b.owner as u64);
            h.write_u64(b.remaining as u64);
        }
        h.finish()
    }

    // ------------------------------------------------------------------
    // Boucle de simulation
    // ------------------------------------------------------------------

    /// Calcule un tour. Les ordres sont appliqués dans l'ordre donné.
    pub fn tick(&mut self, intents: &[(PlayerId, Intent)]) -> &TickUpdate {
        self.update.tick = self.tick;
        self.update.changed_tiles.clear();
        self.update.events.clear();

        if !matches!(self.phase, Phase::Ended { .. }) {
            for (player, intent) in intents {
                self.apply_intent(*player, intent);
            }

            match self.phase {
                Phase::Spawn => self.update_spawn_phase(),
                Phase::Playing => {
                    self.run_population();
                    self.run_buildings();
                    self.run_attacks();
                    self.run_boats();
                    self.check_eliminations();
                    self.check_victory();
                }
                Phase::Ended { .. } => {}
            }
        }

        self.tick += 1;
        &self.update
    }

    fn apply_intent(&mut self, player: PlayerId, intent: &Intent) {
        let alive = self.player(player).map(|p| p.alive);
        let result = match alive {
            None => Err(RejectReason::UnknownPlayer),
            Some(false) => Err(RejectReason::NotAlive),
            Some(true) => match *intent {
                Intent::Spawn { tile } => self.spawn(player, tile),
                Intent::Attack { target, ratio_permille } => {
                    self.launch_attack(player, target, ratio_permille)
                }
                Intent::CancelAttack { attack_id } => self.cancel_attack(player, attack_id),
                Intent::Naval { tile, ratio_permille } => {
                    self.launch_boat(player, tile, ratio_permille)
                }
                Intent::Build { tile, kind } => self.build(player, tile, kind),
                Intent::SetPlan { ref plan } => self.set_plan(player, plan),
                Intent::Research { tech } => self.start_research(player, tech),
            },
        };
        if let Err(reason) = result {
            self.update.events.push(GameEvent::Rejected { player, reason });
        }
    }

    // ------------------------------------------------------------------
    // Fin de partie
    // ------------------------------------------------------------------

    fn check_eliminations(&mut self) {
        let dead: Vec<PlayerId> = self
            .players
            .iter()
            .filter(|p| p.alive && p.spawned && p.tiles == 0)
            .map(|p| p.id)
            .collect();

        for player in dead {
            let p = self.player_mut(player);
            p.alive = false;
            p.workers = 0;
            p.soldiers = [0; crate::defs::UNIT_COUNT];
            p.garrison = 0;
            p.resources = Cost::default();

            // Ses attaques disparaissent, celles qui le visaient s'arrêtent.
            let attacks = std::mem::take(&mut self.attacks);
            for attack in attacks {
                if attack.attacker == player || attack.target == player {
                    self.finish_attack(attack, 1000);
                } else {
                    self.attacks.push(attack);
                }
            }
            self.boats.retain(|b| b.owner != player);

            let by = self.last_conqueror[player as usize - 1];
            self.update.events.push(GameEvent::Eliminated { player, by });
        }
    }

    fn check_victory(&mut self) {
        let alive: Vec<&Player> = self.players.iter().filter(|p| p.alive).collect();

        let winner = if self.players.len() > 1 && alive.len() <= 1 {
            Some(alive.first().map(|p| p.id))
        } else {
            alive
                .iter()
                .find(|p| {
                    p.tiles as u64 * 1000
                        >= self.land_tiles as u64 * self.config.win_land_permille as u64
                })
                .map(|p| Some(p.id))
        };

        if let Some(winner) = winner {
            self.phase = Phase::Ended { winner };
            self.update.events.push(GameEvent::Victory { winner });
        }
    }

    // ------------------------------------------------------------------
    // Utilitaires partagés par les systèmes
    // ------------------------------------------------------------------

    /// Seule fonction qui change le propriétaire d'une case : tient à jour
    /// les compteurs, les bâtiments capturés et la liste des changements.
    pub(crate) fn set_owner(&mut self, tile: TileIdx, new_owner: PlayerId) {
        let old = self.owner[tile as usize];
        if old == new_owner {
            return;
        }
        let terrain = self.map.terrain(tile) as usize;
        if old != NEUTRAL {
            let i = old as usize - 1;
            self.players[i].tiles -= 1;
            self.terrain_tiles[i][terrain] -= 1;
            if new_owner != NEUTRAL {
                self.last_conqueror[i] = Some(new_owner);
            }
        }
        if new_owner != NEUTRAL {
            let i = new_owner as usize - 1;
            self.players[i].tiles += 1;
            self.terrain_tiles[i][terrain] += 1;
        }
        self.owner[tile as usize] = new_owner;
        self.update.changed_tiles.push(tile);

        if self.building_at[tile as usize] != NO_BUILDING {
            self.capture_building(tile, old, new_owner);
        }
    }

    /// Identifiant que recevra le prochain objet (sans le consommer).
    pub(crate) fn peek_id(&self) -> u32 {
        self.next_id
    }

    pub(crate) fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    #[inline]
    pub(crate) fn player_ref(&self, id: PlayerId) -> &Player {
        &self.players[id as usize - 1]
    }

    #[inline]
    pub(crate) fn player_mut(&mut self, id: PlayerId) -> &mut Player {
        &mut self.players[id as usize - 1]
    }
}

/// FNV-1a 64 bits : simple, stable, suffisant pour détecter une
/// désynchronisation.
struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325)
    }

    fn write_u64(&mut self, value: u64) {
        for byte in value.to_le_bytes() {
            self.0 ^= byte as u64;
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}
