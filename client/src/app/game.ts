// Écran de jeu : relie le worker (moteur), le rendu, l'interface et les
// commandes du joueur.

import {
  BUILDINGS,
  DEPOSITS,
  type BuildingKind,
  type FromWorker,
  type GameEvent,
  type Insight,
  type Intent,
  type MapPayload,
  type PopulationPlan,
  type RejectReason,
  type Snapshot,
} from '../engine/protocol';
import { Camera } from '../render/camera';
import { computeAnchors, Overlay, type Label, type Markers } from '../render/overlay';
import { isPassable, PLAYER_COLORS, TERRAIN_LABELS, type Rgb } from '../render/palette';
import { Renderer } from '../render/renderer';
import { formatNumber } from '../ui/format';
import { Hud, type HudSeat } from '../ui/hud';
import { BUILDING_INFO, DEPOSIT_INFO, roman, TECH_INFO } from '../ui/labels';
import { Guide, type Guidance } from './guide';
import type { MenuResult } from './menu';

const HUMAN_ID = 1;
const SPEEDS = [1, 2, 4];
const PAN_SPEED = 900; // pixels physiques par seconde
const DRAG_THRESHOLD = 5;

export class GameScreen {
  readonly el: HTMLElement;
  private renderer: Renderer;
  private overlay: Overlay;
  private hud: Hud;
  private camera = new Camera();
  private colors = new Map<number, Rgb>();

  private map: MapPayload | null = null;
  private owners: Uint16Array | null = null;
  /** Masse terrestre de chaque case (-1 pour l'eau et les montagnes). */
  private regions: Int32Array | null = null;
  private dirtyMin = Infinity;
  private dirtyMax = -1;
  private snapshot: Snapshot | null = null;
  private labels: Label[] = [];
  private anchorsAge = Infinity;

  private ratio = 500;
  /** Aperçus du moteur (estimations d'attaque, curseurs, bâtiments). */
  private insight: Insight | null = null;
  private speedIndex = 0;
  private paused = false;
  private ended = false;
  private eliminated = false;
  private destroyed = false;
  /** Bâtiment en cours de placement. */
  private placing: BuildingKind | null = null;
  private guide: Guide | null = null;
  private guidance: Guidance = { objective: null, alerts: [] };

  private keys = new Set<string>();
  private pointer: { x: number; y: number; startX: number; startY: number; button: number; dragging: boolean } | null =
    null;
  private mouse: [number, number] | null = null;
  private lastFrame = performance.now();
  private fpsFrames = 0;
  private fpsTime = 0;

  constructor(
    private worker: Worker,
    setup: MenuResult,
    private onExit: () => void,
  ) {
    const seats: HudSeat[] = setup.seats.map((s, i) => ({ ...s, color: PLAYER_COLORS[i % PLAYER_COLORS.length] }));
    seats.forEach((s, i) => this.colors.set(i + 1, s.color));

    this.el = document.createElement('div');
    this.el.className = 'game';
    const canvas = document.createElement('canvas');
    const overlayCanvas = document.createElement('canvas');
    overlayCanvas.className = 'overlay';
    this.el.append(canvas, overlayCanvas);

    this.renderer = new Renderer(canvas);
    this.overlay = new Overlay(overlayCanvas);
    this.hud = new Hud(seats, {
      onRatio: (permille) => this.setRatio(permille),
      onFocusPlayer: (id) => this.focusPlayer(id),
      onFocusTile: (tile) => this.focusTile(tile),
      ownerOf: (tile) => this.owners?.[tile] ?? 0,
      onCancelAttack: (attackId) => this.send({ type: 'cancel_attack', attack_id: attackId }),
      onPlan: (plan: PopulationPlan) => this.send({ type: 'set_plan', plan }),
      onResearch: (tech) => this.send({ type: 'research', tech }),
      onSelectBuilding: (kind) => this.setPlacing(kind),
      onHelp: () => this.toggleHelp(),
    });
    this.hud.setRatio(this.ratio);
    this.el.append(this.hud.el);
    this.hud.showBanner('Génération du monde…');

    this.bindInput(canvas);
    requestAnimationFrame(this.frame);
    worker.postMessage({ kind: 'start', setup: setup.setup, human: HUMAN_ID });
    worker.postMessage({ kind: 'ratio', ratio: this.ratio });
  }

  destroy(): void {
    this.destroyed = true;
    this.worker.postMessage({ kind: 'stop' });
    window.removeEventListener('keydown', this.onKeyDown);
    window.removeEventListener('keyup', this.onKeyUp);
    window.removeEventListener('resize', this.onResize);
    this.renderer.dispose();
    this.el.remove();
  }

  /** Messages du worker. */
  handle(msg: FromWorker): void {
    if (this.destroyed) return;
    switch (msg.kind) {
      case 'map':
        this.onMap(msg.map);
        break;
      case 'tick':
        this.onTick(msg.snapshot, msg.changes);
        break;
      case 'insight':
        this.insight = msg.insight;
        this.hud.setInsight(msg.insight);
        break;
      case 'error':
        this.hud.log(`Erreur moteur : ${msg.message}`, 'danger');
        break;
      case 'agents':
        break;
    }
  }

  // ------------------------------------------------------------------ moteur

  private onMap(map: MapPayload): void {
    this.map = map;
    this.owners = map.owners;
    this.regions = computeRegions(map);
    this.guide = new Guide(map, HUMAN_ID);
    this.hud.setRules(map.rules);
    this.renderer.setMap(map);
    this.renderer.setPalette(this.colors);
    const [w, h] = this.renderer.resize();
    this.overlay.resize();
    this.camera.fit(map.width, map.height, w, h);
    this.hud.showBanner(null);
  }

  private onTick(snapshot: Snapshot, changes: Uint32Array): void {
    const map = this.map;
    const owners = this.owners;
    if (!map || !owners) return;

    for (let i = 0; i < changes.length; i += 2) {
      const tile = changes[i];
      owners[tile] = changes[i + 1];
      const row = (tile / map.width) | 0;
      if (row < this.dirtyMin) this.dirtyMin = row;
      if (row > this.dirtyMax) this.dirtyMax = row;
    }

    this.snapshot = snapshot;
    for (const event of snapshot.events) this.onEvent(event, snapshot);

    if (this.guide) {
      this.guidance = this.guide.compute(snapshot, owners, this.insight);
      this.hud.showObjective(this.guidance.objective, this.guidance.alerts);
      if (this.hud.helpOpen) this.hud.showHelp(this.guidance.objective, this.guidance.alerts);
    }
    if (this.placing) {
      const info = BUILDING_INFO[this.placing];
      this.hud.showBanner(`${info.name} : clique sur ton territoire pour la placer · clic droit ou Échap pour annuler`);
    } else {
      this.hud.showBanner(null);
    }

    this.anchorsAge += 1;
    if (this.anchorsAge >= 10) this.refreshLabels();
    else this.updateLabelTroops();

    this.hud.update(snapshot, HUMAN_ID, map.tickMs);
  }

  private onEvent(event: GameEvent, snap: Snapshot): void {
    const name = (id: number) => (id === 0 ? 'les terres libres' : (snap.players.find((p) => p.id === id)?.name ?? '?'));
    const mine = (id: number) => id === HUMAN_ID;
    switch (event.type) {
      case 'game_started':
        this.hud.log(
          'La partie commence ! Étends-toi sur les terres libres, construis, et choisis ta stratégie. Trêve de 30 s.',
          'important',
        );
        break;
      case 'attack_started':
        if (mine(event.attacker)) {
          this.hud.log(`Tu envoies ${formatNumber(event.troops)} soldats vers ${name(event.target)}.`);
        } else if (mine(event.target)) {
          this.hud.log(`${name(event.attacker)} t’attaque avec ${formatNumber(event.troops)} soldats !`, 'danger');
        }
        break;
      case 'attacks_clashed':
        if (mine(event.a) || mine(event.b)) {
          this.hud.log(`Les armées de ${name(event.a)} et ${name(event.b)} se percutent.`);
        }
        break;
      case 'attack_ended':
        if (mine(event.attacker) && event.returned > 0) {
          this.hud.log(`Campagne contre ${name(event.target)} terminée, ${formatNumber(event.returned)} soldats rentrent.`);
        }
        break;
      case 'boat_launched':
        if (mine(event.owner)) {
          this.hud.log(`Un bateau part avec ${formatNumber(event.troops)} soldats.`);
        } else if (this.owners?.[event.tile] === HUMAN_ID) {
          this.hud.log(`${name(event.owner)} envoie un bateau vers tes côtes !`, 'danger');
        }
        break;
      case 'boat_landed':
        if (mine(event.owner)) this.hud.log('Débarquement réussi : tes troupes prennent pied.', 'important');
        break;
      case 'boat_lost':
        if (mine(event.owner)) this.hud.log('Ton bateau n’avait pas assez de troupes pour débarquer.', 'danger');
        break;
      case 'build_started':
        if (mine(event.owner)) this.hud.log(`Chantier lancé : ${BUILDING_INFO[event.kind].name}.`);
        break;
      case 'building_completed':
        if (mine(event.owner)) this.hud.log(`${BUILDING_INFO[event.kind].name} terminée.`, 'important');
        break;
      case 'building_captured': {
        const what = BUILDING_INFO[event.kind].name.toLowerCase();
        if (mine(event.to)) this.hud.log(`Tu prends un(e) ${what} à ${name(event.from)} !`, 'important');
        else if (mine(event.from)) this.hud.log(`${name(event.to)} t’a pris un(e) ${what} !`, 'danger');
        break;
      }
      case 'building_destroyed':
        if (mine(event.owner)) this.hud.log('L’ennemi a détruit un de tes forts.', 'danger');
        break;
      case 'research_started':
        if (mine(event.player)) this.hud.log(`Recherche lancée : ${TECH_INFO[event.tech].name} ${roman(event.level)}.`);
        break;
      case 'research_completed':
        if (mine(event.player)) {
          this.hud.log(`Recherche terminée : ${TECH_INFO[event.tech].name} ${roman(event.level)}.`, 'important');
        }
        break;
      case 'rejected':
        if (mine(event.player)) this.hud.log(rejectMessage(event.reason), 'danger');
        break;
      case 'eliminated':
        this.hud.log(
          `${name(event.player)} a été éliminé${event.by ? ` par ${name(event.by)}` : ''}.`,
          mine(event.player) ? 'danger' : 'important',
        );
        if (mine(event.player) && !this.ended) this.onEliminated(event.by ? name(event.by) : null);
        break;
      case 'victory':
        this.onVictory(event.winner, snap);
        break;
      case 'spawned':
        if (mine(event.player) && this.map) {
          // Zoom sur le point de départ.
          const dpr = window.devicePixelRatio || 1;
          const [w, h] = this.renderer.resize();
          this.camera.centerOn(event.tile % this.map.width, Math.floor(event.tile / this.map.width), 5 * dpr, w, h);
        }
        break;
      case 'attack_reinforced':
        break;
    }
  }

  private onEliminated(by: string | null): void {
    this.eliminated = true;
    this.hud.showModal('ÉLIMINÉ', by ? `${by} a pris tes dernières terres.` : 'Ta civilisation a disparu.', [
      { label: 'Regarder la suite', onClick: () => this.hud.hideModal() },
      { label: 'Menu', primary: true, onClick: () => this.onExit() },
    ]);
  }

  private onVictory(winner: number | null, snap: Snapshot): void {
    this.ended = true;
    const won = winner === HUMAN_ID;
    const winnerName = snap.players.find((p) => p.id === winner)?.name;
    this.hud.showModal(
      won ? 'VICTOIRE' : 'DÉFAITE',
      won ? 'Ta civilisation domine le monde.' : winnerName ? `${winnerName} domine le monde.` : 'Personne ne l’emporte.',
      [
        { label: 'Voir la carte', onClick: () => this.hud.hideModal() },
        { label: 'Nouvelle partie', primary: true, onClick: () => this.onExit() },
      ],
    );
  }

  private refreshLabels(): void {
    const map = this.map;
    const owners = this.owners;
    const snap = this.snapshot;
    if (!map || !owners || !snap) return;
    this.anchorsAge = 0;
    const anchors = computeAnchors(owners, map.width, snap.players.length);
    this.labels = snap.players
      .filter((p) => p.alive && anchors.has(p.id))
      .map((p) => {
        const [x, y] = anchors.get(p.id)!;
        return { x, y, tiles: p.tiles, name: p.name, soldiers: soldiersAt(p), garrison: p.garrison_power, color: this.colors.get(p.id)! };
      });
  }

  private updateLabelTroops(): void {
    const snap = this.snapshot;
    if (!snap) return;
    for (const label of this.labels) {
      const p = snap.players.find((q) => q.name === label.name);
      if (p) {
        label.soldiers = soldiersAt(p);
        label.garrison = p.garrison_power;
        label.tiles = p.tiles;
      }
    }
  }

  // ------------------------------------------------------------------ actions

  private send(intent: Intent): void {
    this.worker.postMessage({ kind: 'intent', intent });
  }

  private tileAt(clientX: number, clientY: number): number | null {
    const map = this.map;
    if (!map) return null;
    const dpr = window.devicePixelRatio || 1;
    const [tx, ty] = this.camera.toTile(clientX * dpr, clientY * dpr);
    const x = Math.floor(tx);
    const y = Math.floor(ty);
    if (x < 0 || y < 0 || x >= map.width || y >= map.height) return null;
    return y * map.width + x;
  }

  private setPlacing(kind: BuildingKind | null): void {
    this.placing = kind;
    this.hud.setSelected(kind);
    this.el.classList.toggle('placing', kind !== null);
  }

  /** Vérification côté interface (le moteur revérifie tout). */
  private placementProblem(tile: number, kind: BuildingKind): string | null {
    const map = this.map;
    const snap = this.snapshot;
    if (!map || !snap || !this.owners) return 'Patiente…';
    if (this.owners[tile] !== HUMAN_ID) return 'Uniquement sur ton territoire.';
    if (!isPassable(map.terrain[tile])) return 'Case infranchissable.';
    if (snap.buildings.some((b) => b.tile === tile)) return 'Case déjà occupée.';
    const def = map.rules.buildings[BUILDINGS.indexOf(kind)];
    if (def.deposit !== 'none' && DEPOSITS[map.deposits[tile]] !== def.deposit) {
      return `À placer sur : ${DEPOSIT_INFO[def.deposit]?.name.toLowerCase()}.`;
    }
    return null;
  }

  private clickTile(tile: number): void {
    const map = this.map;
    const owners = this.owners;
    const snap = this.snapshot;
    if (!map || !owners || !snap || this.ended || this.eliminated) return;

    if (this.placing) {
      const problem = this.placementProblem(tile, this.placing);
      if (problem) {
        this.hud.log(problem, 'danger');
        return;
      }
      this.send({ type: 'build', tile, kind: this.placing });
      if (!this.keys.has('ShiftLeft') && !this.keys.has('ShiftRight')) this.setPlacing(null);
      return;
    }

    if (!isPassable(map.terrain[tile])) {
      this.hud.log('Cette zone est infranchissable (eau ou montagne).');
      return;
    }

    const me = snap.players.find((p) => p.id === HUMAN_ID);
    if (snap.phase.type === 'spawn') {
      if (!me?.spawned) this.send({ type: 'spawn', tile });
      return;
    }

    const target = owners[tile];
    if (target === HUMAN_ID) return;

    // Même masse terrestre que mon territoire : à pied. Sinon : bateau.
    if (this.ownsLandIn(this.regions![tile])) {
      this.send({ type: 'attack', target, ratio_permille: this.ratio });
    } else if (isCoastal(map, tile)) {
      this.send({ type: 'naval', tile, ratio_permille: this.ratio });
    } else {
      this.hud.log('Trop loin dans les terres pour un débarquement : vise une côte.');
    }
  }

  private ownsLandIn(region: number): boolean {
    const owners = this.owners!;
    const regions = this.regions!;
    for (let i = 0; i < owners.length; i++) {
      if (owners[i] === HUMAN_ID && regions[i] === region) return true;
    }
    return false;
  }

  private toggleHelp(): void {
    if (this.hud.helpOpen) this.hud.hideHelp();
    else this.hud.showHelp(this.guidance.objective, this.guidance.alerts);
  }

  private togglePause(): void {
    if (this.ended) return;
    this.paused = !this.paused;
    this.worker.postMessage({ kind: 'pause', paused: this.paused });
    this.updateSpeedText();
  }

  private setSpeed(index: number): void {
    this.speedIndex = index;
    this.worker.postMessage({ kind: 'speed', speed: SPEEDS[index] });
    this.updateSpeedText();
  }

  private updateSpeedText(): void {
    this.hud.setSpeed(this.paused ? 'PAUSE' : `Vitesse ${SPEEDS[this.speedIndex]}x`);
  }

  private askExit(): void {
    if (this.hud.modalOpen) {
      this.hud.hideModal();
      if (this.paused && !this.ended) this.togglePause();
      return;
    }
    if (!this.paused && !this.ended) this.togglePause();
    this.hud.showModal('PAUSE', 'Quitter la partie en cours ?', [
      {
        label: 'Reprendre',
        primary: true,
        onClick: () => {
          this.hud.hideModal();
          if (this.paused) this.togglePause();
        },
      },
      { label: 'Quitter vers le menu', onClick: () => this.onExit() },
    ]);
  }

  // ------------------------------------------------------------------ entrées

  private bindInput(canvas: HTMLCanvasElement): void {
    canvas.addEventListener('contextmenu', (e) => e.preventDefault());

    canvas.addEventListener('pointerdown', (e) => {
      this.hud.hideMenu();
      canvas.setPointerCapture(e.pointerId);
      this.pointer = { x: e.clientX, y: e.clientY, startX: e.clientX, startY: e.clientY, button: e.button, dragging: false };
    });

    canvas.addEventListener('pointermove', (e) => {
      this.mouse = [e.clientX, e.clientY];
      const p = this.pointer;
      if (!p) return;
      if (!p.dragging && Math.hypot(e.clientX - p.startX, e.clientY - p.startY) > DRAG_THRESHOLD) {
        p.dragging = true;
        this.el.classList.add('dragging');
      }
      if (p.dragging) {
        const dpr = window.devicePixelRatio || 1;
        this.camera.pan(-(e.clientX - p.x) * dpr, -(e.clientY - p.y) * dpr);
      }
      p.x = e.clientX;
      p.y = e.clientY;
    });

    canvas.addEventListener('pointerup', (e) => {
      const p = this.pointer;
      this.pointer = null;
      this.el.classList.remove('dragging');
      if (!p || p.dragging) return;
      if (p.button === 2) {
        this.setPlacing(null);
        return;
      }
      const tile = this.tileAt(e.clientX, e.clientY);
      if (tile !== null && p.button === 0) this.clickTile(tile);
    });

    canvas.addEventListener('pointerleave', () => {
      this.mouse = null;
    });

    canvas.addEventListener(
      'wheel',
      (e) => {
        e.preventDefault();
        const dpr = window.devicePixelRatio || 1;
        this.camera.zoomAt(e.clientX * dpr, e.clientY * dpr, Math.exp(-e.deltaY * 0.0015));
      },
      { passive: false },
    );

    window.addEventListener('keydown', this.onKeyDown);
    window.addEventListener('keyup', this.onKeyUp);
    window.addEventListener('resize', this.onResize);
  }

  private onKeyDown = (e: KeyboardEvent): void => {
    if ((e.target as HTMLElement)?.tagName === 'INPUT') return;
    this.keys.add(e.code);
    switch (e.code) {
      case 'Space':
        e.preventDefault();
        this.togglePause();
        return;
      case 'Digit1':
        this.setSpeed(0);
        return;
      case 'Digit2':
        this.setSpeed(1);
        return;
      case 'Digit3':
        this.setSpeed(2);
        return;
      // Positions physiques : A/E en AZERTY, Q/E en QWERTY.
      case 'KeyQ':
        this.adjustRatio(-50);
        return;
      case 'KeyE':
        this.adjustRatio(50);
        return;
      case 'KeyH':
        this.toggleHelp();
        return;
      case 'Escape':
        if (this.hud.helpOpen) this.hud.hideHelp();
        else if (this.placing) this.setPlacing(null);
        else if (this.hud.menuOpen) this.hud.hideMenu();
        else this.askExit();
        return;
    }
    // Raccourcis de construction : lettre affichée (selon la disposition du clavier).
    const letter = e.key.toUpperCase();
    const kind = BUILDINGS.find((k) => BUILDING_INFO[k].key === letter);
    if (kind && this.snapshot?.phase.type === 'playing') this.setPlacing(this.placing === kind ? null : kind);
  };

  private onKeyUp = (e: KeyboardEvent): void => {
    this.keys.delete(e.code);
  };

  private onResize = (): void => {
    this.renderer.resize();
    this.overlay.resize();
  };

  private adjustRatio(delta: number): void {
    this.setRatio(Math.max(50, Math.min(1000, this.ratio + delta)));
  }

  /** Part des soldats envoyés : l'interface et les estimations du moteur suivent. */
  private setRatio(permille: number): void {
    this.ratio = permille;
    this.hud.setRatio(permille);
    this.worker.postMessage({ kind: 'ratio', ratio: permille });
  }

  /** Centre la carte sur le territoire d'un joueur (sans changer le zoom). */
  private focusPlayer(id: number): void {
    const name = this.snapshot?.players.find((p) => p.id === id)?.name;
    const label = this.labels.find((l) => l.name === name);
    if (label) this.focusAt(label.x, label.y);
  }

  private focusTile(tile: number): void {
    if (this.map) this.focusAt(tile % this.map.width, Math.floor(tile / this.map.width));
  }

  private focusAt(x: number, y: number): void {
    const [w, h] = this.renderer.resize();
    this.camera.centerOn(x, y, this.camera.zoom, w, h);
  }

  /** Déplacement au clavier, quelle que soit la disposition (ZQSD, WASD, flèches). */
  private keyboardPan(dt: number): void {
    let dx = 0;
    let dy = 0;
    const k = this.keys;
    const shift = k.has('ShiftLeft') || k.has('ShiftRight') ? 2 : 1;
    if (k.has('KeyW') || k.has('ArrowUp')) dy -= 1;
    if (k.has('KeyS') || k.has('ArrowDown')) dy += 1;
    if (k.has('KeyA') || k.has('ArrowLeft')) dx -= 1;
    if (k.has('KeyD') || k.has('ArrowRight')) dx += 1;
    if (dx || dy) this.camera.pan(dx * PAN_SPEED * shift * dt, dy * PAN_SPEED * shift * dt);
  }

  // ------------------------------------------------------------------ rendu

  private frame = (now: number): void => {
    if (this.destroyed) return;
    requestAnimationFrame(this.frame);
    const dt = Math.min(0.05, (now - this.lastFrame) / 1000);
    this.lastFrame = now;

    this.fpsFrames += 1;
    this.fpsTime += dt;
    if (this.fpsTime >= 0.5) {
      this.hud.setFps(this.fpsFrames / this.fpsTime);
      this.fpsFrames = 0;
      this.fpsTime = 0;
    }

    const map = this.map;
    const [w, h] = this.renderer.resize();
    this.overlay.resize();
    if (map) {
      this.keyboardPan(dt);
      this.camera.clamp(map.width, map.height, w, h);
      if (this.owners && this.dirtyMax >= this.dirtyMin) {
        this.renderer.updateOwners(this.owners, this.dirtyMin, this.dirtyMax);
        this.dirtyMin = Infinity;
        this.dirtyMax = -1;
      }
    }

    const hoverTile = this.mouse && !this.pointer?.dragging ? this.tileAt(this.mouse[0], this.mouse[1]) : null;
    this.updateHover(hoverTile);
    this.renderer.draw(this.camera);

    let hover: [number, number] | null = null;
    let hoverColor = 'rgba(245, 235, 205, 0.8)';
    if (map && hoverTile !== null) {
      hover = [hoverTile % map.width, Math.floor(hoverTile / map.width)];
      if (this.placing) {
        hoverColor = this.placementProblem(hoverTile, this.placing) ? 'rgba(224, 110, 95, 0.95)' : 'rgba(140, 220, 130, 0.95)';
      }
    }
    const focus = this.placing
      ? (this.map?.rules.buildings[BUILDINGS.indexOf(this.placing)]?.deposit ?? 'none')
      : (this.guidance.objective?.deposit ?? null);
    this.overlay.draw(
      this.camera,
      this.labels,
      this.markers(),
      hover,
      hoverColor,
      focus === 'none' ? (this.placing ? 'all' : null) : focus,
    );
  };

  private markers(): Markers {
    const map = this.map;
    const snap = this.snapshot;
    if (!map) return { buildings: [], boats: [], deposits: [] };
    const at = (tile: number): [number, number] => [(tile % map.width) + 0.5, Math.floor(tile / map.width) + 0.5];
    const color = (id: number): Rgb => this.colors.get(id) ?? [128, 128, 128];
    const deposits: Markers['deposits'] = [];
    for (let i = 0; i < map.depositSites.length; i += 2) {
      const [x, y] = at(map.depositSites[i]);
      deposits.push({ x, y, kind: DEPOSITS[map.depositSites[i + 1]] });
    }
    if (!snap) return { buildings: [], boats: [], deposits };
    return {
      deposits,
      buildings: snap.buildings.map((b) => {
        const [x, y] = at(b.tile);
        const total = map.rules.buildings[BUILDINGS.indexOf(b.kind)]?.build_ticks ?? 1;
        return {
          x,
          y,
          glyph: BUILDING_INFO[b.kind].glyph,
          color: color(b.owner),
          progress: b.remaining > 0 ? 1 - b.remaining / total : 1,
        };
      }),
      boats: snap.boats.map((b) => {
        const [x, y] = at(b.tile);
        const [tx, ty] = at(b.destination);
        return { x, y, tx, ty, color: color(b.owner), troops: b.troops };
      }),
    };
  }

  private updateHover(tile: number | null): void {
    const map = this.map;
    const owners = this.owners;
    if (!map || !owners || tile === null || !this.mouse) {
      this.renderer.setHoverOwner(0);
      this.hud.showTooltip(0, 0, null);
      return;
    }
    const owner = owners[tile];
    this.renderer.setHoverOwner(this.placing ? 0 : owner);
    if (this.placing) {
      const problem = this.placementProblem(tile, this.placing);
      this.hud.showTooltip(this.mouse[0], this.mouse[1], problem ?? `Construire : ${BUILDING_INFO[this.placing].name}`);
      return;
    }
    const terrain = TERRAIN_LABELS[map.terrain[tile]] ?? '?';
    const river = map.river[tile] > 0 ? ' · rivière' : '';
    const deposit = DEPOSIT_INFO[DEPOSITS[map.deposits[tile]]];
    const building = this.snapshot?.buildings.find((b) => b.tile === tile);
    const extra =
      (deposit ? ` · ${deposit.name}` : '') +
      (building ? ` · ${BUILDING_INFO[building.kind].name}${building.remaining > 0 ? ' (chantier)' : ''}` : '');
    const ownerName =
      owner === 0 ? 'Terres libres' : (this.snapshot?.players.find((p) => p.id === owner)?.name ?? '?');
    if (!isPassable(map.terrain[tile])) {
      this.hud.showTooltip(this.mouse[0], this.mouse[1], terrain);
      return;
    }
    const lines = [`${ownerName} · ${terrain}${river}${extra}`];
    const output = building && building.remaining === 0 ? this.buildingOutput(building.kind, building.owner) : '';
    if (output) lines.push(output);
    // Aperçu de l'attaque : ce que donnerait un clic ici avec l'envoi actuel.
    let tone: 'good' | 'mid' | 'bad' | undefined;
    const estimate = owner !== HUMAN_ID ? this.insight?.estimates.find((e) => e.target === owner) : undefined;
    if (estimate && owner === 0) {
      lines.push(`⚔ ${formatNumber(estimate.sent)} envoyés → ≈ ${formatNumber(estimate.tiles)} cases`);
    } else if (estimate) {
      const share = estimate.share_permille / 10;
      tone = estimate.conquers_all || share >= 30 ? 'good' : share >= 8 ? 'mid' : 'bad';
      const verdict = estimate.conquers_all
        ? 'tout son territoire'
        : `≈ ${formatNumber(estimate.tiles)} cases (${share < 1 ? '< 1' : Math.round(share)} % de ses terres)`;
      lines.push(`⚔ ${formatNumber(estimate.power)} contre 🛡 ${formatNumber(estimate.defense)}`, `→ ${verdict}`);
    }
    this.hud.showTooltip(this.mouse[0], this.mouse[1], lines.join('\n'), tone);
  }

  /** Production d'un bâtiment posé, quand tous ses emplois sont pourvus. */
  private buildingOutput(kind: BuildingKind, owner: number): string {
    const i = BUILDINGS.indexOf(kind);
    const def = this.map?.rules.buildings[i];
    const job = this.map?.rules.buildings[i] ? BUILDING_JOB[kind] : undefined;
    if (!def || !job) return def && def.housing > 0 ? `☺ +${formatNumber(def.housing)} places` : '';
    // Les miens : bonus de l'Académie compris ; ceux des autres : rendement de base.
    const mine = owner === HUMAN_ID ? this.insight?.gains[i] : undefined;
    const perSecond = mine ? mine.output[job.key] : (def.jobs * def.job_yield_milli * 10) / 1000;
    return `${job.icon} jusqu’à +${formatNumber(perSecond)}/s · ${def.jobs} emplois`;
  }
}

/** Ressource produite par les bâtiments à emplois. */
const BUILDING_JOB: Partial<Record<BuildingKind, { key: 'food' | 'wood' | 'stone' | 'iron'; icon: string }>> = {
  farm: { key: 'food', icon: '◆' },
  sawmill: { key: 'wood', icon: '▲' },
  quarry: { key: 'stone', icon: '■' },
  mine: { key: 'iron', icon: '⬣' },
};

function soldiersAt(p: { soldiers: number[] }): number {
  return p.soldiers.reduce((a, b) => a + b, 0);
}

function rejectMessage(reason: RejectReason): string {
  switch (reason) {
    case 'no_border':
      return 'Pas de frontière commune avec cette cible : étends-toi jusqu’à elle.';
    case 'not_enough_troops':
      return 'Pas assez de soldats pour cette attaque (augmente la part de soldats).';
    case 'tile_taken':
      return 'Cette case est déjà occupée.';
    case 'invalid_tile':
      return 'Impossible ici.';
    case 'wrong_phase':
      return 'Action impossible pour l’instant.';
    case 'not_enough_resources':
      return 'Pas assez de ressources.';
    case 'too_close':
      return 'Trop près d’un bâtiment du même type.';
    case 'not_owned':
      return 'Cette case ne t’appartient pas.';
    case 'too_many_boats':
      return 'Tous tes bateaux sont déjà en mer.';
    case 'no_path':
      return 'Aucune route maritime vers cette côte depuis tes rivages.';
    case 'immune':
      return 'Trêve de départ : on ne peut pas encore attaquer les autres joueurs.';
    case 'max_count':
      return 'Tu ne peux en avoir qu’un seul.';
    case 'no_academy':
      return 'Il faut d’abord construire une Académie.';
    case 'max_level':
      return 'Cette recherche est déjà au niveau maximum.';
    case 'locked':
      return 'Cette recherche demande d’abord autre chose (voir le cadenas).';
    case 'needs_deposit':
      return 'Ce bâtiment doit être placé sur le bon gisement.';
    case 'researching':
      return 'Une recherche est déjà en cours.';
    default:
      return 'Ordre refusé.';
  }
}

function isCoastal(map: MapPayload, tile: number): boolean {
  const x = tile % map.width;
  const y = Math.floor(tile / map.width);
  const water = (t: number) => map.terrain[t] <= 1;
  return (
    (y > 0 && water(tile - map.width)) ||
    (y < map.height - 1 && water(tile + map.width)) ||
    (x > 0 && water(tile - 1)) ||
    (x < map.width - 1 && water(tile + 1))
  );
}

/** Masses terrestres connexes (même règle que le moteur : 4 voisins franchissables). */
function computeRegions(map: MapPayload): Int32Array {
  const { width, height, terrain } = map;
  const regions = new Int32Array(width * height).fill(-1);
  const stack: number[] = [];
  let next = 0;
  for (let start = 0; start < regions.length; start++) {
    if (regions[start] !== -1 || !isPassable(terrain[start])) continue;
    regions[start] = next;
    stack.push(start);
    while (stack.length) {
      const t = stack.pop()!;
      const x = t % width;
      const y = (t / width) | 0;
      const visit = (n: number) => {
        if (regions[n] === -1 && isPassable(terrain[n])) {
          regions[n] = next;
          stack.push(n);
        }
      };
      if (y > 0) visit(t - width);
      if (x < width - 1) visit(t + 1);
      if (y < height - 1) visit(t + width);
      if (x > 0) visit(t - 1);
    }
    next++;
  }
  return regions;
}
