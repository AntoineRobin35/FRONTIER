// Types échangés avec le moteur Rust (JSON) et avec le worker.
// Ils reflètent les types serde de crates/engine et crates/session :
// toute modification côté Rust doit être répercutée ici.

export type PlayerId = number;

/** Dans l'ordre du moteur (crates/engine/src/defs.rs). */
export type BuildingKind =
  | 'city'
  | 'barracks'
  | 'farm'
  | 'sawmill'
  | 'archery'
  | 'stable'
  | 'quarry'
  | 'fort'
  | 'mine'
  | 'academy';
export const BUILDINGS: BuildingKind[] = [
  'city',
  'barracks',
  'farm',
  'sawmill',
  'archery',
  'stable',
  'quarry',
  'fort',
  'mine',
  'academy',
];

/** Technologies de l'Académie, dans l'ordre du moteur (crates/engine/src/defs.rs). */
export type TechId =
  | 'agriculture'
  | 'forestry'
  | 'masonry'
  | 'metallurgy'
  | 'architecture'
  | 'urbanism'
  | 'granaries'
  | 'drill'
  | 'blades'
  | 'bowmanship'
  | 'horsemanship'
  | 'shields'
  | 'discipline'
  | 'instructors'
  | 'colonization'
  | 'logistics'
  | 'fortification';
export const TECHS: TechId[] = [
  'agriculture',
  'forestry',
  'masonry',
  'metallurgy',
  'architecture',
  'urbanism',
  'granaries',
  'drill',
  'blades',
  'bowmanship',
  'horsemanship',
  'shields',
  'discipline',
  'instructors',
  'colonization',
  'logistics',
  'fortification',
];

export interface Research {
  tech: TechId;
  level: number;
  remaining: number;
  total: number;
}

export type Deposit = 'none' | 'fertile' | 'grove' | 'quarry' | 'iron';
export const DEPOSITS: Deposit[] = ['none', 'fertile', 'grove', 'quarry', 'iron'];

export interface Cost {
  food: number;
  wood: number;
  stone: number;
  iron: number;
}

export interface PopulationPlan {
  soldiers_permille: number;
  garrison_permille: number;
  /** Guerriers, archers, cavaliers. */
  unit_mix: [number, number, number];
  /** Cueilleurs, bûcherons, mineurs, forgerons ; null = automatique. */
  job_mix: [number, number, number, number] | null;
}

export type Intent =
  | { type: 'spawn'; tile: number }
  | { type: 'attack'; target: PlayerId; ratio_permille: number }
  | { type: 'cancel_attack'; attack_id: number }
  | { type: 'naval'; tile: number; ratio_permille: number }
  | { type: 'build'; tile: number; kind: BuildingKind }
  | { type: 'set_plan'; plan: PopulationPlan }
  | { type: 'research'; tech: TechId };

export type Difficulty = 'easy' | 'normal';
export type MapSize = 'small' | 'medium' | 'large';

export type Controller = { kind: 'human' } | { kind: 'bot'; agent: string; difficulty: Difficulty };

export interface Seat {
  name: string;
  controller: Controller;
}

export interface SessionSetup {
  seed: number;
  map_size: MapSize;
  seats: Seat[];
}

export interface AgentInfo {
  id: string;
  name: string;
  description: string;
}

export type Phase = { type: 'spawn' } | { type: 'playing' } | { type: 'ended'; winner: PlayerId | null };

/** Personnes et ressources en unités ; production et consommation par seconde. */
export interface PlayerView {
  id: PlayerId;
  name: string;
  alive: boolean;
  spawned: boolean;
  tiles: number;
  techs: number[];
  research: Research | null;
  /** Coût (unités) du prochain niveau ; null au maximum. */
  tech_costs: (Cost | null)[];
  tech_ticks: number[];
  /** Prérequis manquant (technologie ou bâtiment). */
  tech_locked: boolean[];
  population: number;
  housing: number;
  workers: number;
  jobs: [number, number, number, number];
  job_slots: [number, number, number, number];
  soldiers: [number, number, number];
  away: number;
  garrison: number;
  garrison_training: number;
  defense: number;
  resources: Cost;
  storage: number;
  production: Cost;
  food_consumption: number;
  /** Dépense par seconde pour former soldats et garnison. */
  training_spend: Cost;
  training_rate: number;
  plan: PopulationPlan;
  unit_unlocked: [boolean, boolean, boolean];
  buildings: number[];
  build_costs: Cost[];
}

export interface BoatView {
  id: number;
  owner: PlayerId;
  tile: number;
  destination: number;
  troops: number;
}

export interface BuildingView {
  id: number;
  kind: BuildingKind;
  tile: number;
  owner: PlayerId;
  remaining: number;
}

export interface AttackView {
  id: number;
  attacker: PlayerId;
  target: PlayerId;
  troops: number;
}

export type RejectReason =
  | 'wrong_phase'
  | 'unknown_player'
  | 'not_alive'
  | 'already_spawned'
  | 'invalid_tile'
  | 'tile_taken'
  | 'invalid_target'
  | 'not_enough_troops'
  | 'no_border'
  | 'unknown_attack'
  | 'not_owned'
  | 'too_close'
  | 'not_enough_resources'
  | 'too_many_boats'
  | 'no_path'
  | 'immune'
  | 'needs_deposit'
  | 'researching'
  | 'invalid_plan'
  | 'max_count'
  | 'no_academy'
  | 'max_level'
  | 'locked';

export type GameEvent =
  | { type: 'spawned'; player: PlayerId; tile: number }
  | { type: 'game_started' }
  | { type: 'attack_started'; attack_id: number; attacker: PlayerId; target: PlayerId; troops: number }
  | { type: 'attack_reinforced'; attack_id: number; attacker: PlayerId; target: PlayerId; troops: number }
  | { type: 'attacks_clashed'; a: PlayerId; b: PlayerId; troops: number }
  | { type: 'attack_ended'; attack_id: number; attacker: PlayerId; target: PlayerId; returned: number }
  | { type: 'boat_launched'; boat_id: number; owner: PlayerId; tile: number; troops: number }
  | { type: 'boat_landed'; boat_id: number; owner: PlayerId; tile: number }
  | { type: 'boat_lost'; boat_id: number; owner: PlayerId }
  | { type: 'build_started'; building_id: number; owner: PlayerId; kind: BuildingKind; tile: number }
  | { type: 'building_completed'; building_id: number; owner: PlayerId; kind: BuildingKind; tile: number }
  | { type: 'building_captured'; building_id: number; kind: BuildingKind; from: PlayerId; to: PlayerId }
  | { type: 'building_destroyed'; building_id: number; kind: BuildingKind; owner: PlayerId }
  | { type: 'research_started'; player: PlayerId; tech: TechId; level: number }
  | { type: 'research_completed'; player: PlayerId; tech: TechId; level: number }
  | { type: 'rejected'; player: PlayerId; reason: RejectReason }
  | { type: 'eliminated'; player: PlayerId; by: PlayerId | null }
  | { type: 'victory'; winner: PlayerId | null };

export interface Snapshot {
  tick: number;
  phase: Phase;
  land_tiles: number;
  immunity: boolean;
  players: PlayerView[];
  attacks: AttackView[];
  boats: BoatView[];
  buildings: BuildingView[];
  events: GameEvent[];
}

/** Sous-ensemble des règles (GameConfig) utilisé par l'interface. */
export interface RulesConfig {
  immunity_ticks: number;
  units: {
    requires: BuildingKind | null;
    cost_milli: Cost;
    attack_permille: number;
    defense_permille: number;
    speed_permille: number;
  }[];
  buildings: {
    build_ticks: number;
    max_count: number | null;
    deposit: Deposit;
    housing: number;
    jobs: number;
    job_yield_milli: number;
    min_distance: number;
  }[];
  techs: {
    branch: 'economy' | 'military';
    per_level_permille: number;
    max_level: number;
    requires: [TechId, number] | null;
    requires_building: BuildingKind | null;
  }[];
  job_requires: (BuildingKind | null)[];
  fort_radius: number;
  garrison_training_max: number;
  boat_wood: number;
}

export interface MapPayload {
  seed: number;
  width: number;
  height: number;
  tickMs: number;
  terrain: Uint8Array;
  elevation: Uint16Array;
  river: Uint8Array;
  rivers: Float32Array;
  deposits: Uint8Array;
  depositSites: Uint32Array;
  owners: Uint16Array;
  rules: RulesConfig;
}

export type ToWorker =
  | { kind: 'hello' }
  | { kind: 'start'; setup: SessionSetup; human: PlayerId }
  | { kind: 'intent'; intent: Intent }
  | { kind: 'speed'; speed: number }
  | { kind: 'pause'; paused: boolean }
  | { kind: 'stop' };

export type FromWorker =
  | { kind: 'agents'; agents: AgentInfo[] }
  | { kind: 'map'; map: MapPayload }
  | { kind: 'tick'; snapshot: Snapshot; changes: Uint32Array }
  | { kind: 'error'; message: string };
