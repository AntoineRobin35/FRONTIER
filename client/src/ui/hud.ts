// Interface en jeu.
//
// Principes (voir docs/RECHERCHE_UX.md) :
// - n'afficher que ce qui sert maintenant : une ressource, une ligne ou un
//   bâtiment apparaît quand il devient utile (dévoilement progressif) ;
// - chaque donnée porte une petite info-bulle (attribut `data-tip`) qui dit
//   simplement ce que c'est ;
// - les nombres ne sont redessinés que quelques fois par seconde ;
// - la touche H ouvre l'aide « que faire maintenant ? ».

import {
  BUILDINGS,
  TECHS,
  type AttackView,
  type BuildingKind,
  type PlayerView,
  type PopulationPlan,
  type RulesConfig,
  type Snapshot,
  type TechId,
} from '../engine/protocol';
import { rgbCss, type Rgb } from '../render/palette';
import type { Objective } from '../app/guide';
import { escapeHtml, formatNumber, formatTime } from './format';
import { BUILDING_INFO, JOB_NAMES, RESOURCES, TECH_INFO, UNIT_NAMES, canPay, costText, roman } from './labels';

export interface HudSeat {
  name: string;
  agentName: string | null;
  color: Rgb;
}

export interface MenuOption {
  label: string;
  detail: string;
  enabled: boolean;
  onClick: () => void;
}

export interface ModalAction {
  label: string;
  primary?: boolean;
  onClick: () => void;
}

export interface HudCallbacks {
  onRatio: (permille: number) => void;
  onCancelAttack: (attackId: number) => void;
  onPlan: (plan: PopulationPlan) => void;
  onResearch: (tech: TechId) => void;
  onSelectBuilding: (kind: BuildingKind | null) => void;
  onHelp: () => void;
}

/** Un conseil de l'aide : un titre court et une phrase. */
export interface HelpTip {
  title: string;
  text: string;
  urgent?: boolean;
}

type LogKind = 'normal' | 'important' | 'danger';

const LOG_LIMIT = 6;
const LOG_LIFETIME_MS = 9000;
/** Après un réglage, on n'écrase pas les curseurs pendant ce délai. */
const EDIT_GRACE_MS = 1500;
/** Rafraîchissement des nombres : assez pour suivre, pas assez pour clignoter. */
const RENDER_INTERVAL_MS = 300;

/** Métier qui produit chaque ressource (index dans `jobs`). */
const RESOURCE_JOB: Record<'food' | 'wood' | 'stone' | 'iron', { index: number; name: string }> = {
  food: { index: 0, name: 'cueilleurs' },
  wood: { index: 1, name: 'bûcherons' },
  stone: { index: 2, name: 'mineurs' },
  iron: { index: 3, name: 'forgerons' },
};

const RESOURCE_TIPS: Record<string, string> = {
  food: 'Nourriture : nourrit ta population. Si elle manque, la population baisse. Un excédent la fait grandir.',
  wood: 'Bois : sert à construire et à former les soldats.',
  stone: 'Pierre : extraite par les mineurs (carrière). Sert aux bâtiments avancés.',
  iron: 'Fer : produit par les forgerons (mine). Sert aux constructions avancées.',
};

export class Hud {
  readonly el: HTMLElement;
  private top: HTMLElement;
  private leaderboard: HTMLElement;
  private popStats: HTMLElement;
  private details: HTMLElement;
  private attacksEl: HTMLElement;
  private buildBar: HTMLElement;
  private logEl: HTMLElement;
  private banner: HTMLElement;
  private tooltipEl: HTMLElement;
  private tipEl: HTMLElement;
  private helpEl: HTMLElement;
  private academyEl: HTMLElement;
  private lastSnap: { snap: Snapshot; meId: number } | null = null;
  private objectiveEl: HTMLElement;
  private objectiveKey = '';
  private objectiveHidden = false;
  private objectiveId: string | null = null;
  private freshTimer: ReturnType<typeof setTimeout> | undefined;
  private highlighted: string | null = null;
  private modalEl: HTMLElement;
  private menuEl: HTMLElement;
  private ratioInput: HTMLInputElement;
  private ratioLabel: HTMLElement;
  private seats: HudSeat[];
  private rules: RulesConfig | null = null;
  private speedText = 'Vitesse 1x';
  private plan: PopulationPlan | null = null;
  private editedAt = 0;
  private detailed = false;
  private selected: BuildingKind | null = null;
  private lastRender = 0;
  private lastBuildKey = '';
  private pressing = false;

  constructor(seats: HudSeat[], private cb: HudCallbacks) {
    this.seats = seats;
    this.el = document.createElement('div');
    this.el.className = 'hud';
    this.el.innerHTML = `
      <div class="topbar panel"></div>
      <div class="leaderboard panel"></div>
      <div class="pop-panel panel">
        <div class="pop-stats"></div>
        <div class="slider-row main-slider"
          data-tip="Part de ta population qui devient soldat. Les autres travaillent et produisent. Former des soldats prend du temps et coûte de la nourriture et du bois.">
          <label>Soldats</label>
          <input type="range" min="0" max="80" step="1" data-plan="soldiers" />
          <b data-value="soldiers"></b>
        </div>
        <button class="btn btn-small toggle-details"
          data-tip="Régler toi-même les métiers, la composition de l'armée et la garnison.">⚙ Réglages détaillés</button>
        <div class="details hidden"></div>
        <div class="slider-row ratio"
          data-tip="Part de tes soldats envoyés quand tu cliques pour attaquer ou t'étendre (touches A / E).">
          <label>Envoi</label>
          <input type="range" min="5" max="100" step="5" value="50" />
          <b>50%</b>
        </div>
        <div class="attacks"></div>
      </div>
      <div class="objective panel hidden"></div>
      <div class="build-bar panel"></div>
      <div class="log"></div>
      <div class="banner panel hidden"></div>
      <div class="tooltip hidden"></div>
      <div class="tip hidden"></div>
      <div class="help-panel panel hidden"></div>
      <div class="academy-panel panel hidden"></div>
      <div class="build-menu panel hidden"></div>
      <div class="modal-backdrop hidden"><div class="modal panel"></div></div>`;

    this.top = this.el.querySelector('.topbar')!;
    this.leaderboard = this.el.querySelector('.leaderboard')!;
    this.popStats = this.el.querySelector('.pop-stats')!;
    this.details = this.el.querySelector('.details')!;
    this.attacksEl = this.el.querySelector('.attacks')!;
    this.buildBar = this.el.querySelector('.build-bar')!;
    this.logEl = this.el.querySelector('.log')!;
    this.banner = this.el.querySelector('.banner')!;
    this.tooltipEl = this.el.querySelector('.tooltip')!;
    this.tipEl = this.el.querySelector('.tip')!;
    this.helpEl = this.el.querySelector('.help-panel')!;
    this.academyEl = this.el.querySelector('.academy-panel')!;
    this.academyEl.addEventListener('pointerdown', (e) => {
      const target = e.target as HTMLElement;
      if (target.closest('[data-close]')) this.hideAcademy();
      const btn = target.closest<HTMLButtonElement>('[data-research]');
      if (btn && !btn.disabled) this.cb.onResearch(btn.dataset.research as TechId);
    });
    this.objectiveEl = this.el.querySelector('.objective')!;
    this.objectiveEl.addEventListener('click', (e) => {
      const target = e.target as HTMLElement;
      if (target.closest('[data-hide]')) {
        this.objectiveHidden = !this.objectiveHidden;
        this.objectiveKey = '';
      }
      if (target.closest('[data-help]')) this.cb.onHelp();
    });
    this.modalEl = this.el.querySelector('.modal-backdrop')!;
    this.menuEl = this.el.querySelector('.build-menu')!;
    this.ratioInput = this.el.querySelector('.ratio input')!;
    this.ratioLabel = this.el.querySelector('.ratio b')!;

    this.ratioInput.addEventListener('input', () => {
      const permille = Number(this.ratioInput.value) * 10;
      this.cb.onRatio(permille);
      this.setRatio(permille);
    });
    this.attacksEl.addEventListener('pointerdown', (e) => {
      const btn = (e.target as HTMLElement).closest<HTMLElement>('[data-cancel]');
      if (btn) this.cb.onCancelAttack(Number(btn.dataset.cancel));
    });
    this.el.querySelector('.toggle-details')!.addEventListener('click', () => {
      this.detailed = !this.detailed;
      this.details.classList.toggle('hidden', !this.detailed);
      this.editedAt = 0;
      this.lastRender = 0;
    });
    const panel = this.el.querySelector('.pop-panel')!;
    // Pendant qu'on tient un curseur, on ne redessine pas le panneau.
    panel.addEventListener('pointerdown', () => (this.pressing = true));
    window.addEventListener('pointerup', () => (this.pressing = false));
    panel.addEventListener('input', (e) => this.onPlanInput(e));
    panel.addEventListener('change', (e) => this.onPlanInput(e));
    this.top.addEventListener('pointerdown', (e) => {
      const target = e.target as HTMLElement;
      if (target.closest('[data-academy]')) this.toggleAcademy();
      if (target.closest('[data-help]')) this.cb.onHelp();
    });
    this.helpEl.addEventListener('click', (e) => {
      if ((e.target as HTMLElement).closest('[data-close]')) this.hideHelp();
    });
    this.buildBar.addEventListener('click', (e) => {
      const btn = (e.target as HTMLElement).closest<HTMLButtonElement>('[data-build]');
      if (!btn || btn.disabled) return;
      const kind = btn.dataset.build as BuildingKind;
      this.setSelected(this.selected === kind ? null : kind);
      this.cb.onSelectBuilding(this.selected);
    });

    // Petites info-bulles : tout élément avec `data-tip`.
    this.el.addEventListener('mouseover', (e) => {
      const target = (e.target as HTMLElement).closest<HTMLElement>('[data-tip]');
      if (target) this.showTip(target);
      else this.tipEl.classList.add('hidden');
    });
    this.el.addEventListener('mouseleave', () => this.tipEl.classList.add('hidden'));
  }

  setRules(rules: RulesConfig): void {
    this.rules = rules;
  }

  setRatio(permille: number): void {
    const pct = Math.round(permille / 10);
    this.ratioInput.value = String(pct);
    this.ratioLabel.textContent = `${pct}%`;
  }

  setFps(_fps: number): void {
    // Le compteur d'images n'est plus affiché : c'est une info de développeur.
  }

  setSpeed(text: string): void {
    this.speedText = text;
    this.lastRender = 0;
  }

  setSelected(kind: BuildingKind | null): void {
    this.selected = kind;
    for (const btn of this.buildBar.querySelectorAll<HTMLElement>('[data-build]')) {
      btn.classList.toggle('active', btn.dataset.build === kind);
    }
  }

  // ------------------------------------------------------------------ info-bulles

  private showTip(target: HTMLElement): void {
    const text = target.dataset.tip;
    if (!text) return;
    this.tipEl.textContent = text;
    this.tipEl.classList.remove('hidden');
    const r = target.getBoundingClientRect();
    const tip = this.tipEl.getBoundingClientRect();
    let top = r.bottom + 6;
    if (top + tip.height > window.innerHeight - 8) top = r.top - tip.height - 6;
    const left = Math.max(8, Math.min(r.left, window.innerWidth - tip.width - 8));
    this.tipEl.style.left = `${left}px`;
    this.tipEl.style.top = `${Math.max(8, top)}px`;
  }

  // ------------------------------------------------------------------ mise à jour

  update(snap: Snapshot, meId: number, tickMs: number): void {
    const now = performance.now();
    if (now - this.lastRender < RENDER_INTERVAL_MS) return;
    this.lastRender = now;
    const me = snap.players.find((p) => p.id === meId);
    this.lastSnap = { snap, meId };
    this.renderTop(snap, me, tickMs);
    if (me && this.academyOpen) this.renderAcademy(me);
    this.renderLeaderboard(snap, meId);
    if (me) {
      this.renderPopulation(me);
      this.renderAttacks(me, snap);
      this.renderBuildBar(me);
    }
    this.applyHighlight();
  }

  private renderTop(snap: Snapshot, me: PlayerView | undefined, tickMs: number): void {
    let academy = '';
    if (me && me.buildings[BUILDINGS.indexOf('academy')] > 0) {
      const r = me.research;
      const status = r
        ? `${TECH_INFO[r.tech].name} ${roman(r.level)} · ${Math.round((1 - r.remaining / r.total) * 100)}%`
        : 'aucune recherche';
      academy = `<button class="btn btn-small academy-btn ${r ? '' : 'idle'}" data-academy
        data-tip="Ouvrir l'Académie : lancer des recherches qui améliorent ton économie et ton armée.">✦ Académie · ${escapeHtml(status)}</button>`;
    }

    // Dévoilement progressif : une ressource n'apparaît que si on en a ou en produit.
    const resources =
      me && snap.phase.type !== 'spawn'
        ? RESOURCES.filter((r) => r.key === 'food' || r.key === 'wood' || me.resources[r.key] > 0 || me.production[r.key] > 0)
            .map((r) => {
              const amount = me.resources[r.key];
              const full = amount >= me.storage * 0.98;
              // Vrai solde : production − consommation − formation des soldats.
              const eaten = r.key === 'food' ? me.food_consumption : 0;
              const training = me.training_spend[r.key];
              const rate = me.production[r.key] - eaten - training;
              const sign = rate >= 0 ? '+' : '';
              const job = RESOURCE_JOB[r.key];
              const workers = me.jobs[job.index];
              const parts = [`produit +${formatNumber(me.production[r.key])}/s (${formatNumber(workers)} ${job.name})`];
              if (eaten > 0) parts.push(`mangé −${formatNumber(eaten)}/s`);
              if (training > 0) parts.push(`formation des soldats −${formatNumber(training)}/s`);
              const why =
                workers === 0 && r.key === 'wood' && me.jobs[0] > 0
                  ? ' Aucun bûcheron : tous tes travailleurs cueillent pour nourrir la population (un soldat mange 1,5 fois plus qu’un travailleur).'
                  : '';
              const tip = `${RESOURCE_TIPS[r.key]} Solde ${sign}${formatNumber(rate)}/s : ${parts.join(', ')}.${why} Stock ${formatNumber(amount)} sur ${formatNumber(me.storage)} max${full ? ' : réserve pleine, la production se perd' : ''}.`;
              return `<div class="res ${full ? 'full' : ''}" data-tip="${escapeHtml(tip)}">
                <span class="res-icon" style="color:${r.color}">${r.icon}</span>
                <b>${formatNumber(amount)}</b><small class="${rate < 0 ? 'neg' : ''}">${sign}${formatNumber(rate)}/s</small>
              </div>`;
            })
            .join('')
        : '';

    const pop =
      me && snap.phase.type !== 'spawn'
        ? `<div class="res" data-tip="Population / place maximale. Elle grandit si la nourriture est en excédent. Plus de territoire et de villes = plus de place.">
            <span class="res-icon">☺</span><b>${formatNumber(me.population)}</b><small>/ ${formatNumber(me.housing)}</small></div>`
        : '';

    const immunity =
      snap.immunity && snap.phase.type === 'playing'
        ? '<span class="immune" data-tip="Au début de la partie, personne ne peut attaquer un autre joueur. Profites-en pour t\'étendre.">Trêve</span>'
        : '';
    setHtml(this.top, `
      <div class="top-left"><b>FRONTIER</b>
        <span data-tip="Temps de jeu.">${formatTime(snap.tick * tickMs)}</span>
        <span data-tip="Espace : pause · 1, 2, 3 : vitesse.">${escapeHtml(this.speedText)}</span>${immunity}</div>
      ${academy}
      <div class="resources">${resources}${pop}</div>
      <button class="btn btn-small help-btn" data-help data-tip="Que faire maintenant ? Conseils selon ta situation.">? Aide (H)</button>`);
  }

  private renderLeaderboard(snap: Snapshot, meId: number): void {
    const land = Math.max(1, snap.land_tiles);
    const players = [...snap.players].sort((a, b) => Number(b.alive) - Number(a.alive) || b.tiles - a.tiles);
    setHtml(
      this.leaderboard,
      `<h2 data-tip="Toutes les civilisations, de la plus grande à la plus petite. Contrôle 80 % des terres pour gagner.">CIVILISATIONS</h2>` +
      players
        .map((p) => {
          const seat = this.seats[p.id - 1];
          const sub = seat?.agentName ? `IA ${escapeHtml(seat.agentName)}` : 'Toi';
          const home = homeForce(p);
          return `
          <div class="lb-row ${p.alive ? '' : 'dead'} ${p.id === meId ? 'mine' : ''}">
            <span class="swatch" style="background:${rgbCss(seat?.color ?? [128, 128, 128])}"></span>
            <span class="name">${escapeHtml(p.name)}<br><span class="sub">${sub}</span></span>
            <span class="num" data-tip="Part des terres du monde qu'elle contrôle.">${((p.tiles * 100) / land).toFixed(1)}%</span>
            <span class="num" data-tip="Soldats qui défendent son territoire (soldats à la maison + garnison).">⚔ ${formatNumber(home)}</span>
          </div>`;
        })
        .join(''),
    );
  }

  private renderPopulation(me: PlayerView): void {
    const soldiers = me.soldiers.reduce((a, b) => a + b, 0);
    const rows: string[] = [
      row('Travailleurs', formatNumber(me.workers), 'Ils produisent nourriture, bois et le reste. Plus ils sont nombreux, plus ton économie tourne.'),
      row('Soldats à la maison', formatNumber(soldiers), 'Ils défendent ton territoire et partent quand tu attaques.'),
    ];
    if (me.away > 0) {
      rows.push(row('En campagne', formatNumber(me.away), 'Soldats partis en attaque ou en bateau. Ils ne défendent pas ta maison.'));
    }
    if (me.garrison > 0 || me.plan.garrison_permille > 0) {
      rows.push(
        row(
          'Garnison',
          `${formatNumber(me.garrison)} · ×${(me.garrison_training / 1000).toFixed(1)}`,
          "Défenseurs : ils n'attaquent jamais, mais s'entraînent avec le temps (×1 → ×3) et défendent de mieux en mieux.",
        ),
      );
    }
    rows.push(
      row(
        'Force de défense',
        formatNumber(me.defense),
        'Puissance totale qui protège ton territoire. Plus elle est haute, plus les attaques ennemies ralentissent et coûtent cher.',
      ),
    );
    setHtml(this.popStats, `
      <h2><span class="swatch" style="display:inline-block;background:${rgbCss(this.seats[me.id - 1].color)}"></span>
        ${escapeHtml(me.name)}</h2>${rows.join('')}`);

    const editing = performance.now() - this.editedAt < EDIT_GRACE_MS;
    if (!editing) this.plan = structuredClone(me.plan);
    const plan = this.plan!;
    const main = this.el.querySelector<HTMLInputElement>('[data-plan="soldiers"]')!;
    if (!editing) main.value = String(Math.round(plan.soldiers_permille / 10));
    this.el.querySelector('[data-value="soldiers"]')!.textContent = `${Math.round(plan.soldiers_permille / 10)}%`;

    if (this.detailed && !editing && !this.pressing) this.renderDetails(me, plan);
  }

  private renderDetails(me: PlayerView, plan: PopulationPlan): void {
    const auto = plan.job_mix === null;
    const jobMix = plan.job_mix ?? [5, 5, 5, 5];
    const jobTips = [
      'Récoltent la nourriture (plaines, marais, côtes ; beaucoup plus dans une ferme).',
      'Coupent le bois (forêts ; beaucoup plus dans une scierie).',
      'Extraient la pierre (collines ; beaucoup plus dans une carrière).',
      'Produisent le fer (uniquement dans une mine).',
    ];
    // Dévoilement : on ne montre que les métiers déjà accessibles.
    const jobs = JOB_NAMES.map((name, i) => {
      const unlock = this.rules?.job_requires[i];
      if (unlock && me.buildings[BUILDINGS.indexOf(unlock)] === 0) return '';
      return `
        <div class="slider-row" data-tip="${escapeHtml(jobTips[i] + ' Nombre au travail / emplois disponibles.')}">
          <label>${name}</label>
          <input type="range" min="0" max="10" step="1" value="${Math.round(jobMix[i])}" data-job="${i}" ${auto ? 'disabled' : ''} />
          <b>${formatNumber(me.jobs[i])}<small>/${formatNumber(me.job_slots[i])}</small></b>
        </div>`;
    }).join('');
    const unitTips = [
      'Guerriers : polyvalents, efficaces contre les forts.',
      'Archers : excellents en défense et en forêt.',
      'Cavaliers : rapides et forts en plaine, faibles en forêt et en marais.',
    ];
    const units = UNIT_NAMES.map((name, i) => {
      if (!me.unit_unlocked[i]) return '';
      return `
        <div class="slider-row" data-tip="${escapeHtml(unitTips[i] + ' Le curseur règle leur part dans tes nouveaux soldats.')}">
          <label>${name}</label>
          <input type="range" min="0" max="10" step="1" value="${Math.round(plan.unit_mix[i] / 100)}" data-unit="${i}" />
          <b>${formatNumber(me.soldiers[i])}</b>
        </div>`;
    }).join('');
    const lockedUnits = UNIT_NAMES.filter((_, i) => !me.unit_unlocked[i]).length;
    setHtml(this.details, `
      <div class="section-title" data-tip="En automatique, les travailleurs vont d'abord là où il faut nourrir tout le monde, puis vers le bois et le reste.">Métiers
        <label class="check"><input type="checkbox" data-auto ${auto ? 'checked' : ''}/> auto</label></div>
      ${jobs}
      <div class="section-title">Armée</div>
      ${units}
      ${lockedUnits ? `<div class="hint">${lockedUnits === 2 ? 'Archers et cavaliers' : "D'autres unités"} se débloquent plus tard (archerie, écurie).</div>` : ''}
      <div class="section-title">Garnison</div>
      <div class="slider-row" data-tip="Part de ta population en garnison : des défenseurs qui ne quittent jamais ton territoire et deviennent plus forts avec le temps. Stratégie défensive.">
        <label>Garnison</label>
        <input type="range" min="0" max="60" step="1" value="${Math.round(plan.garrison_permille / 10)}" data-plan="garrison" />
        <b>${Math.round(plan.garrison_permille / 10)}%</b>
      </div>`);
  }

  private onPlanInput(e: Event): void {
    const input = e.target as HTMLInputElement;
    if (!this.plan || input.closest('.ratio')) return;
    const plan = this.plan;
    if (input.dataset.plan === 'soldiers') {
      plan.soldiers_permille = Number(input.value) * 10;
    } else if (input.dataset.plan === 'garrison') {
      plan.garrison_permille = Number(input.value) * 10;
    } else if (input.dataset.unit !== undefined) {
      plan.unit_mix[Number(input.dataset.unit)] = Number(input.value) * 100;
    } else if (input.dataset.job !== undefined) {
      const mix = plan.job_mix ?? [5, 5, 5, 5];
      mix[Number(input.dataset.job)] = Number(input.value);
      plan.job_mix = mix;
    } else if (input.dataset.auto !== undefined) {
      plan.job_mix = input.checked ? null : [5, 5, 5, 5];
    } else {
      return;
    }
    if (plan.soldiers_permille + plan.garrison_permille > 1000) {
      plan.garrison_permille = 1000 - plan.soldiers_permille;
    }
    this.editedAt = performance.now();
    this.el.querySelector('[data-value="soldiers"]')!.textContent = `${Math.round(plan.soldiers_permille / 10)}%`;
    const value = input.closest('.slider-row')?.querySelector('b');
    if (input.dataset.plan === 'garrison' && value) value.textContent = `${Math.round(plan.garrison_permille / 10)}%`;
    if (e.type === 'change' || input.type === 'checkbox') {
      if (input.type === 'checkbox') {
        this.editedAt = 0;
        this.lastRender = 0;
      }
      this.cb.onPlan(structuredClone(plan));
    }
  }

  private renderAttacks(me: PlayerView, snap: Snapshot): void {
    const mine = snap.attacks.filter((a) => a.attacker === me.id);
    setHtml(
      this.attacksEl,
      mine.length ? `<div class="section-title">Attaques en cours</div>${mine.map((a) => this.attackRow(a, snap)).join('')}` : '',
    );
  }

  private attackRow(a: AttackView, snap: Snapshot): string {
    const target = a.target === 0 ? 'Terres libres' : (snap.players.find((p) => p.id === a.target)?.name ?? '?');
    const tip =
      a.target === 0
        ? 'Rappeler : les soldats restants rentrent à la maison.'
        : 'Battre en retraite : les soldats rentrent, mais tu en perds 25 %.';
    return `
      <div class="attack-row">
        <span class="grow">→ ${escapeHtml(target)}</span>
        <span data-tip="Soldats encore engagés dans cette attaque.">${formatNumber(a.troops)}</span>
        <button class="btn btn-small" data-cancel="${a.id}" data-tip="${tip}">Rappeler</button>
      </div>`;
  }

  private renderBuildBar(me: PlayerView): void {
    // Dévoilement : on cache ce qui demande de la pierre ou du fer tant qu'on
    // n'en a jamais eu, et l'Académie une fois construite.
    const knows = (key: 'stone' | 'iron') => me.resources[key] > 0 || me.production[key] > 0;
    const available = BUILDINGS.map((kind, i) => ({ kind, i })).filter(({ kind, i }) => {
      const cost = me.build_costs[i];
      if (cost.stone > 0 && !knows('stone')) return false;
      if (cost.iron > 0 && !knows('iron')) return false;
      const max = this.rules?.buildings[i]?.max_count;
      if (max != null && me.buildings[i] >= max && kind === 'academy') return false;
      return true;
    });
    const key =
      available.map(({ kind, i }) => `${kind}:${canPay(me.resources, me.build_costs[i])}:${me.buildings[i]}:${costText(me.build_costs[i])}`).join('|') +
      this.selected;
    if (key === this.lastBuildKey) return;
    this.lastBuildKey = key;
    this.buildBar.innerHTML = available
      .map(({ kind, i }) => {
        const info = BUILDING_INFO[kind];
        const def = this.rules?.buildings[i];
        const cost = me.build_costs[i];
        const affordable = canPay(me.resources, cost);
        const where = def && def.deposit !== 'none' ? ' À placer sur un gisement.' : '';
        const time = def ? ` Chantier : ${Math.round(def.build_ticks / 10)} s.` : '';
        const tip = `${info.name} — ${info.effect}${where}${time} Coût : ${costText(cost)}. Touche ${info.key}.`;
        return `
        <button class="build-btn ${this.selected === kind ? 'active' : ''} ${affordable ? '' : 'poor'}"
          data-build="${kind}" data-tip="${escapeHtml(tip)}">
          <span class="glyph">${info.glyph}</span>
          <span class="bname">${info.name}${me.buildings[i] ? ` <small>×${me.buildings[i]}</small>` : ''}</span>
          <span class="bcost">${escapeHtml(costText(cost)) || 'gratuit'}</span>
        </button>`;
      })
      .join('');
  }

  // ------------------------------------------------------------------ Académie

  get academyOpen(): boolean {
    return !this.academyEl.classList.contains('hidden');
  }

  toggleAcademy(): void {
    if (this.academyOpen) {
      this.hideAcademy();
      return;
    }
    this.academyEl.classList.remove('hidden');
    const last = this.lastSnap;
    const me = last?.snap.players.find((p) => p.id === last.meId);
    if (me) this.renderAcademy(me);
  }

  hideAcademy(): void {
    this.academyEl.classList.add('hidden');
  }

  /** Panneau de l'Académie : deux colonnes, une ligne par recherche. */
  private renderAcademy(me: PlayerView): void {
    const column = (branch: 'economy' | 'military') =>
      TECHS.map((tech, i) => ({ tech, i }))
        .filter(({ i }) => this.rules?.techs[i]?.branch === branch)
        .map(({ tech, i }) => this.techRow(me, tech, i))
        .join('');
    const r = me.research;
    const current = r
      ? `<div class="research-now">En cours : <b>${TECH_INFO[r.tech].name} ${roman(r.level)}</b>
          <span class="obj-progress"><span style="width:${Math.round((1 - r.remaining / r.total) * 100)}%"></span></span>
          ${Math.ceil(r.remaining / 10)} s</div>`
      : `<div class="research-now dim">Aucune recherche en cours : choisis-en une. Une seule à la fois.</div>`;
    setHtml(this.academyEl, `
      <div class="help-head"><h2>✦ ACADÉMIE</h2><button class="btn btn-small" data-close>Fermer</button></div>
      ${current}
      <div class="tech-columns">
        <div><div class="section-title">Économie</div>${column('economy')}</div>
        <div><div class="section-title">Militaire</div>${column('military')}</div>
      </div>
      <div class="hint">Tu ne pourras pas tout rechercher dans une partie : choisis ce qui sert ta stratégie.
        Le niveau 2 coûte aussi de la pierre, le niveau 3 du fer.</div>`);
  }

  private techRow(me: PlayerView, tech: TechId, i: number): string {
    const info = TECH_INFO[tech];
    const def = this.rules?.techs[i];
    const level = me.techs[i] ?? 0;
    const max = def?.max_level ?? 3;
    const dots = Array.from({ length: max }, (_, k) => (k < level ? '●' : '○')).join('');
    const cost = me.tech_costs[i];
    const researching = me.research?.tech === tech;
    let action: string;
    if (!cost) {
      action = '<span class="tech-state">Max</span>';
    } else if (researching) {
      action = '<span class="tech-state">En cours…</span>';
    } else if (me.tech_locked[i]) {
      const need = def?.requires
        ? `${TECH_INFO[def.requires[0]].name} ${roman(def.requires[1])}`
        : def?.requires_building
          ? BUILDING_INFO[def.requires_building].name
          : '';
      action = `<span class="tech-state" data-tip="Il faut d'abord : ${escapeHtml(need)}.">🔒 ${escapeHtml(need)}</span>`;
    } else {
      const ok = canPay(me.resources, cost) && !me.research;
      const why = me.research ? 'Une recherche est déjà en cours.' : canPay(me.resources, cost) ? '' : 'Pas assez de ressources.';
      action = `<button class="btn btn-small" data-research="${tech}" ${ok ? '' : 'disabled'}
        data-tip="${escapeHtml(`Niveau ${roman(level + 1)} : ${costText(cost)} · ${Math.round(me.tech_ticks[i] / 10)} s. ${why}`)}">
        ${escapeHtml(costText(cost))} · ${Math.round(me.tech_ticks[i] / 10)} s</button>`;
    }
    return `
      <div class="tech-row ${researching ? 'active' : ''}">
        <div class="tech-name"><b>${info.name}</b> <span class="tech-dots">${dots}</span></div>
        <div class="tech-effect">${info.effect}</div>
        <div class="tech-action">${action}</div>
      </div>`;
  }

  // ------------------------------------------------------------------ objectif

  /**
   * Bande « Objectif » : discrète, une ligne (titre + progression). La raison
   * s'affiche quelques secondes quand l'objectif change, puis au survol.
   * Seules les alertes urgentes s'ajoutent d'elles-mêmes.
   */
  showObjective(objective: Objective | null, alerts: HelpTip[]): void {
    const urgent = alerts.find((a) => a.urgent) ?? null;
    const key = JSON.stringify([objective, urgent, this.objectiveHidden]);
    if (key === this.objectiveKey) return;
    if (objective?.id !== this.objectiveId) {
      // Nouvel objectif : on montre l'explication un court instant.
      this.objectiveId = objective?.id ?? null;
      this.objectiveEl.classList.add('fresh');
      clearTimeout(this.freshTimer);
      this.freshTimer = setTimeout(() => this.objectiveEl.classList.remove('fresh'), 9000);
    }
    this.objectiveKey = key;
    this.setHighlight(this.objectiveHidden ? null : (objective?.highlight ?? null));
    if (!objective && !urgent) {
      this.objectiveEl.classList.add('hidden');
      return;
    }
    this.objectiveEl.classList.remove('hidden');
    if (this.objectiveHidden) {
      this.objectiveEl.innerHTML = `<button class="obj-mini" data-hide data-tip="Réafficher l'objectif.">🎯</button>`;
      return;
    }
    const progress = objective?.progress
      ? `<span class="obj-progress"><span style="width:${Math.min(100, (objective.progress[0] / objective.progress[1]) * 100).toFixed(0)}%"></span></span>
         <span class="obj-count">${formatNumber(objective.progress[0])}/${formatNumber(objective.progress[1])}</span>`
      : '';
    const alert = urgent
      ? `<div class="obj-alert" data-tip="${escapeHtml(urgent.text)}">⚠ ${escapeHtml(urgent.title)} — ${escapeHtml(urgent.text)}</div>`
      : '';
    this.objectiveEl.innerHTML = `
      ${
        objective
          ? `<div class="obj-line">
              <span class="obj-label">🎯</span>
              <span class="obj-title">${escapeHtml(objective.title)}</span>
              ${progress}
              <button class="obj-mini" data-help data-tip="Tous les conseils et les commandes (touche H).">?</button>
              <button class="obj-mini" data-hide data-tip="Masquer l'objectif.">×</button>
            </div>
            <div class="obj-why">${escapeHtml(objective.why)}</div>`
          : ''
      }
      ${alert}`;
  }

  /** Fait clignoter l'élément de l'interface concerné par l'objectif. */
  private setHighlight(target: string | null): void {
    this.highlighted = target;
    this.applyHighlight();
  }

  private applyHighlight(): void {
    this.el.querySelectorAll('.pulse').forEach((el) => el.classList.remove('pulse'));
    const t = this.highlighted;
    if (!t) return;
    const el =
      t === 'academy-open'
        ? this.top.querySelector('[data-academy]')
        : t === 'soldiers'
          ? this.el.querySelector('.main-slider')
          : this.buildBar.querySelector(`[data-build="${t}"]`);
    el?.classList.add('pulse');
  }

  // ------------------------------------------------------------------ aide (H)

  get helpOpen(): boolean {
    return !this.helpEl.classList.contains('hidden');
  }

  showHelp(objective: Objective | null, alerts: HelpTip[]): void {
    const tips: HelpTip[] = [
      ...alerts.filter((a) => a.urgent),
      ...(objective ? [{ title: `Objectif : ${objective.title}`, text: objective.why }] : []),
      ...alerts.filter((a) => !a.urgent),
    ];
    const items = tips
      .map((t) => `<li class="${t.urgent ? 'urgent' : ''}"><b>${escapeHtml(t.title)}</b><span>${escapeHtml(t.text)}</span></li>`)
      .join('');
    this.helpEl.innerHTML = `
      <div class="help-head"><h2>QUE FAIRE MAINTENANT ?</h2><button class="btn btn-small" data-close>Fermer (H)</button></div>
      <ul class="help-tips">${items}</ul>
      <div class="help-keys">
        <b>Commandes</b>
        <span>Clic : s'étendre / attaquer la zone visée (bateau si c'est une autre île)</span>
        <span>Glisser ou ZQSD : déplacer la carte · Molette : zoom</span>
        <span>A / E : part des soldats envoyés · Espace : pause · 1, 2, 3 : vitesse</span>
        <span>Barre du bas ou lettre : construire · clic droit / Échap : annuler</span>
        <span>✦ Académie (bouton en haut, une fois construite) : recherches</span>
        <span>Survole n'importe quel nombre pour savoir ce que c'est.</span>
      </div>`;
    this.helpEl.classList.remove('hidden');
  }

  hideHelp(): void {
    this.helpEl.classList.add('hidden');
  }

  // ------------------------------------------------------------------ divers

  log(text: string, kind: LogKind = 'normal'): void {
    const entry = document.createElement('div');
    entry.className = `log-entry ${kind === 'normal' ? '' : kind}`;
    entry.textContent = text;
    this.logEl.appendChild(entry);
    while (this.logEl.children.length > LOG_LIMIT) this.logEl.firstElementChild?.remove();
    setTimeout(() => {
      entry.style.opacity = '0';
      setTimeout(() => entry.remove(), 700);
    }, LOG_LIFETIME_MS);
  }

  showBanner(text: string | null): void {
    this.banner.classList.toggle('hidden', !text);
    if (text && this.banner.textContent !== text) this.banner.textContent = text;
  }

  showTooltip(x: number, y: number, text: string | null): void {
    this.tooltipEl.classList.toggle('hidden', !text);
    if (!text) return;
    this.tooltipEl.textContent = text;
    this.tooltipEl.style.left = `${x}px`;
    this.tooltipEl.style.top = `${y}px`;
  }

  showMenu(x: number, y: number, title: string, options: MenuOption[]): void {
    this.menuEl.innerHTML = `<div class="menu-title">${escapeHtml(title)}</div>`;
    for (const option of options) {
      const btn = document.createElement('button');
      btn.className = 'btn menu-option';
      btn.disabled = !option.enabled;
      btn.innerHTML = `<span>${escapeHtml(option.label)}</span><small>${escapeHtml(option.detail)}</small>`;
      btn.addEventListener('click', () => {
        this.hideMenu();
        option.onClick();
      });
      this.menuEl.appendChild(btn);
    }
    this.menuEl.classList.remove('hidden');
    const rect = this.menuEl.getBoundingClientRect();
    this.menuEl.style.left = `${Math.max(8, Math.min(x, window.innerWidth - rect.width - 8))}px`;
    this.menuEl.style.top = `${Math.max(8, Math.min(y, window.innerHeight - rect.height - 8))}px`;
  }

  hideMenu(): void {
    this.menuEl.classList.add('hidden');
  }

  get menuOpen(): boolean {
    return !this.menuEl.classList.contains('hidden');
  }

  get modalOpen(): boolean {
    return !this.modalEl.classList.contains('hidden');
  }

  showModal(title: string, text: string, actions: ModalAction[]): void {
    const modal = this.modalEl.querySelector('.modal')!;
    modal.innerHTML = `<h2>${escapeHtml(title)}</h2><p>${escapeHtml(text)}</p><div class="actions"></div>`;
    const actionsEl = modal.querySelector('.actions')!;
    for (const action of actions) {
      const btn = document.createElement('button');
      btn.className = action.primary ? 'btn btn-primary' : 'btn';
      btn.textContent = action.label;
      btn.addEventListener('click', action.onClick);
      actionsEl.appendChild(btn);
    }
    this.modalEl.classList.remove('hidden');
  }

  hideModal(): void {
    this.modalEl.classList.add('hidden');
  }
}

/** Une ligne « libellé : valeur » avec son info-bulle. */
function row(label: string, value: string, tip: string): string {
  return `<div class="stat" data-tip="${escapeHtml(tip)}"><span>${label}</span><b>${value}</b></div>`;
}

/** Soldats qui défendent réellement le territoire : à la maison + garnison. */
export function homeForce(p: { soldiers: number[]; garrison: number }): number {
  return p.soldiers.reduce((a, b) => a + b, 0) + p.garrison;
}

/** Remplace le contenu d'un élément seulement s'il a changé (évite de perdre
 * un clic pendant un redessin, et le scintillement). */
function setHtml(el: HTMLElement, html: string): void {
  if ((el as HTMLElement & { _html?: string })._html === html) return;
  (el as HTMLElement & { _html?: string })._html = html;
  el.innerHTML = html;
}
