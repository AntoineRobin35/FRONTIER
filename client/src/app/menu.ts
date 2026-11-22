// Menu de création de partie : carte, adversaires et choix de leur IA.

import type { AgentInfo, Difficulty, MapSize, SessionSetup } from '../engine/protocol';
import { CIV_NAMES, PLAYER_COLORS, rgbCss } from '../render/palette';
import { escapeHtml } from '../ui/format';
import { Tutorial } from './tutorial';

export interface SeatInfo {
  name: string;
  /** Nom de l'IA, ou null pour l'humain. */
  agentName: string | null;
}

export interface MenuResult {
  setup: SessionSetup;
  seats: SeatInfo[];
}

interface Opponent {
  agent: string;
  difficulty: Difficulty;
}

const MAX_OPPONENTS = PLAYER_COLORS.length - 1;
const STORAGE_KEY = 'frontier.menu.v1';
const SEEN_KEY = 'frontier.tutorial.seen';

interface Saved {
  name: string;
  mapSize: MapSize;
  opponents: Opponent[];
}

function load(): Partial<Saved> {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}');
  } catch {
    return {};
  }
}

function save(data: Saved): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
  } catch {
    // stockage indisponible : sans importance
  }
}

function randomSeed(): number {
  return Math.floor(Math.random() * 2 ** 31);
}

export class Menu {
  readonly el: HTMLElement;
  private agents: AgentInfo[] = [];
  private opponents: Opponent[] = [];
  private onStart: (result: MenuResult) => void;
  private tutorial: Tutorial;

  constructor(onStart: (result: MenuResult) => void) {
    this.onStart = onStart;
    const saved = load();
    this.el = document.createElement('div');
    this.el.className = 'menu';
    this.tutorial = new Tutorial(() => this.practice());
    this.el.innerHTML = `
      <form class="menu-card panel">
        <h1>FRONTIER</h1>
        <p class="tagline">Des civilisations, une carte, des frontières qui bougent.</p>

        <div class="field-row">
          <label class="field">
            <span>Ton nom</span>
            <input type="text" name="name" maxlength="20" value="${escapeHtml(saved.name ?? 'Joueur')}" />
          </label>
          <label class="field">
            <span>Taille du monde</span>
            <select name="mapSize">
              <option value="small">Petit (480 × 360)</option>
              <option value="medium">Moyen (720 × 540)</option>
              <option value="large">Grand (960 × 720)</option>
            </select>
          </label>
        </div>

        <div class="field-row">
          <label class="field">
            <span>Graine du monde</span>
            <div class="seed-input">
              <input type="number" name="seed" min="0" step="1" value="${randomSeed()}" />
              <button type="button" class="btn" data-action="reroll" title="Nouvelle graine">Aléatoire</button>
            </div>
          </label>
        </div>

        <div class="section-title">
          <span>Adversaires</span>
          <button type="button" class="btn btn-small" data-action="add">+ Ajouter</button>
        </div>
        <div class="opponents"></div>
        <div class="agent-help"></div>

        <div class="menu-error hidden"></div>
        <div class="menu-actions">
          <button type="button" class="btn" data-action="tutorial">Comment jouer</button>
          <button type="button" class="btn" data-action="practice"
            title="Petite carte, un seul adversaire qui ne fait rien : pour découvrir le jeu tranquillement.">S’entraîner</button>
          <button type="submit" class="btn btn-primary">JOUER</button>
        </div>
      </form>`;

    const form = this.el.querySelector('form')!;
    (form.elements.namedItem('mapSize') as HTMLSelectElement).value = saved.mapSize ?? 'medium';
    this.opponents = saved.opponents?.length
      ? saved.opponents
      : [
          { agent: 'colon', difficulty: 'easy' },
          { agent: 'opportuniste', difficulty: 'easy' },
          { agent: 'conquerant', difficulty: 'easy' },
        ];

    form.addEventListener('click', (e) => {
      const target = (e.target as HTMLElement).closest<HTMLElement>('[data-action]');
      if (!target) return;
      const action = target.dataset.action;
      if (action === 'tutorial') {
        this.tutorial.open();
      } else if (action === 'practice') {
        this.practice();
      } else if (action === 'reroll') {
        (form.elements.namedItem('seed') as HTMLInputElement).value = String(randomSeed());
      } else if (action === 'add' && this.opponents.length < MAX_OPPONENTS) {
        const agent = this.agents[this.opponents.length % Math.max(1, this.agents.length - 1)]?.id ?? 'colon';
        this.opponents.push({ agent, difficulty: 'easy' });
        this.renderOpponents();
      } else if (action === 'remove') {
        this.opponents.splice(Number(target.dataset.index), 1);
        this.renderOpponents();
      }
    });

    form.addEventListener('change', (e) => {
      const input = e.target as HTMLSelectElement;
      const index = Number(input.dataset.index);
      if (Number.isNaN(index) || !this.opponents[index]) return;
      if (input.dataset.field === 'agent') this.opponents[index].agent = input.value;
      if (input.dataset.field === 'difficulty') this.opponents[index].difficulty = input.value as Difficulty;
      this.renderHelp(input.dataset.field === 'agent' ? input.value : null);
    });

    form.addEventListener('submit', (e) => {
      e.preventDefault();
      this.submit(form);
    });
    this.renderOpponents();
    this.el.appendChild(this.tutorial.el);
    // Première visite : on propose le tutoriel une fois.
    try {
      if (!localStorage.getItem(SEEN_KEY)) {
        localStorage.setItem(SEEN_KEY, '1');
        setTimeout(() => this.tutorial.open(), 300);
      }
    } catch {
      // stockage indisponible : pas grave
    }
  }

  /** Partie d'entraînement : petite carte, un adversaire passif. */
  private practice(): void {
    const form = this.el.querySelector('form')!;
    const name = (form.elements.namedItem('name') as HTMLInputElement).value.trim() || 'Joueur';
    this.onStart({
      setup: {
        seed: randomSeed(),
        map_size: 'small',
        seats: [
          { name, controller: { kind: 'human' } },
          { name: CIV_NAMES[0], controller: { kind: 'bot', agent: 'passif', difficulty: 'easy' } },
        ],
      },
      seats: [
        { name, agentName: null },
        { name: CIV_NAMES[0], agentName: 'Passif · entraînement' },
      ],
    });
  }

  setAgents(agents: AgentInfo[]): void {
    this.agents = agents;
    const known = new Set(agents.map((a) => a.id));
    for (const o of this.opponents) {
      if (!known.has(o.agent)) o.agent = agents[0]?.id ?? o.agent;
    }
    this.renderOpponents();
  }

  showError(message: string): void {
    const el = this.el.querySelector('.menu-error')!;
    el.textContent = message;
    el.classList.toggle('hidden', !message);
  }

  private renderOpponents(): void {
    const list = this.el.querySelector('.opponents')!;
    const agentOptions = (selected: string) =>
      this.agents
        .map((a) => `<option value="${a.id}" ${a.id === selected ? 'selected' : ''}>${escapeHtml(a.name)}</option>`)
        .join('') || `<option value="${escapeHtml(selected)}">…</option>`;
    list.innerHTML = this.opponents
      .map(
        (o, i) => `
        <div class="opponent">
          <span class="swatch" style="background:${rgbCss(PLAYER_COLORS[i + 1])}"></span>
          <span>${CIV_NAMES[i % CIV_NAMES.length]}</span>
          <select data-index="${i}" data-field="agent">${agentOptions(o.agent)}</select>
          <select class="diff" data-index="${i}" data-field="difficulty">
            <option value="easy" ${o.difficulty === 'easy' ? 'selected' : ''}>Facile</option>
            <option value="normal" ${o.difficulty === 'normal' ? 'selected' : ''}>Normal</option>
          </select>
          <button type="button" class="btn btn-small" data-action="remove" data-index="${i}" title="Retirer"
            ${this.opponents.length <= 1 ? 'disabled' : ''}>✕</button>
        </div>`,
      )
      .join('');
    const add = this.el.querySelector<HTMLButtonElement>('[data-action="add"]')!;
    add.disabled = this.opponents.length >= MAX_OPPONENTS;
    this.renderHelp(null);
  }

  private renderHelp(agentId: string | null): void {
    const help = this.el.querySelector('.agent-help')!;
    const agent = this.agents.find((a) => a.id === agentId);
    if (this.agents.length === 0) {
      help.textContent = 'Chargement du moteur…';
      return;
    }
    help.textContent = agent
      ? `${agent.name} — ${agent.description}`
      : 'Choisis l’IA de chaque adversaire. D’autres IA viendront s’ajouter à cette liste.';
  }

  private submit(form: HTMLFormElement): void {
    const name = (form.elements.namedItem('name') as HTMLInputElement).value.trim() || 'Joueur';
    const mapSize = (form.elements.namedItem('mapSize') as HTMLSelectElement).value as MapSize;
    const seedValue = Number((form.elements.namedItem('seed') as HTMLInputElement).value);
    const seed = Number.isFinite(seedValue) && seedValue >= 0 ? Math.floor(seedValue) % 2 ** 32 : randomSeed();
    save({ name, mapSize, opponents: this.opponents });

    const agentName = (id: string) => this.agents.find((a) => a.id === id)?.name ?? id;
    const seats: SeatInfo[] = [{ name, agentName: null }];
    const setup: SessionSetup = {
      seed,
      map_size: mapSize,
      seats: [{ name, controller: { kind: 'human' } }],
    };
    this.opponents.forEach((o, i) => {
      const civ = CIV_NAMES[i % CIV_NAMES.length];
      setup.seats.push({ name: civ, controller: { kind: 'bot', agent: o.agent, difficulty: o.difficulty } });
      seats.push({ name: civ, agentName: `${agentName(o.agent)} · ${o.difficulty === 'easy' ? 'facile' : 'normal'}` });
    });
    this.onStart({ setup, seats });
  }
}
