// Guide « que faire maintenant ? ».
//
// Inspiré de la recherche (docs/RECHERCHE_UX.md) :
// - le jeu dit toujours clairement quoi faire : UN objectif actif à la fois,
//   avec la raison et la progression (quêtes principales de Rise of Kingdoms,
//   objectifs de Kingdoms and Castles) ;
// - les alertes urgentes (famine, attaque…) passent avant l'objectif ;
// - le joueur garde la main : il peut masquer la carte d'objectif.
//
// Le guide ne fait que lire l'état : il ne joue jamais à la place du joueur.

import { BUILDINGS, DEPOSITS, type BuildingKind, type Deposit, type Insight, type MapPayload, type PlayerView, type Snapshot } from '../engine/protocol';
import { formatNumber } from '../ui/format';
import type { HelpTip } from '../ui/hud';
import { canPay } from '../ui/labels';

export interface Objective {
  id: string;
  title: string;
  why: string;
  progress?: [number, number];
  /** Élément de l'interface à faire clignoter. */
  highlight?: BuildingKind | 'academy-open' | 'soldiers';
  /** Gisements à mettre en évidence sur la carte. */
  deposit?: Deposit;
}

export interface Guidance {
  objective: Objective | null;
  alerts: HelpTip[];
}

/** Ce que le guide sait de la carte, recalculé une fois par seconde environ. */
interface Survey {
  /** Gisements libres (sans bâtiment) dans mon territoire, par type. */
  freeDeposits: Record<Deposit, number>;
}

const EXPAND_GOAL = 600;
const POPULATION_GOAL = 2_000;

export class Guide {
  private done = new Set<string>();
  private survey: Survey | null = null;
  private surveyAge = Infinity;

  constructor(
    private map: MapPayload,
    private me: number,
  ) {}

  /** Objectifs terminés (pour ne pas les redonner). */
  isDone(id: string): boolean {
    return this.done.has(id);
  }

  compute(snap: Snapshot, owners: Uint16Array, insight: Insight | null = null): Guidance {
    const me = snap.players.find((p) => p.id === this.me);
    if (!me) return { objective: null, alerts: [] };
    if (snap.phase.type === 'spawn') {
      return {
        objective: me.spawned
          ? { id: 'wait', title: 'Les autres civilisations se placent…', why: 'La partie commence dans un instant.' }
          : {
              id: 'spawn',
              title: 'Choisis ton point de départ',
              why: 'Clique sur une terre. Les plaines et forêts nourrissent bien ; les icônes ◆ ▲ ■ sont des gisements précieux.',
            },
        alerts: [],
      };
    }

    this.surveyAge += 1;
    if (this.surveyAge >= 10 || !this.survey) {
      this.survey = this.computeSurvey(snap, owners);
      this.surveyAge = 0;
    }
    return { objective: this.nextObjective(snap, me, this.survey, insight), alerts: this.alerts(snap, me) };
  }

  private computeSurvey(snap: Snapshot, owners: Uint16Array): Survey {
    const freeDeposits: Record<Deposit, number> = { none: 0, fertile: 0, grove: 0, quarry: 0, iron: 0 };
    const built = new Set(snap.buildings.map((b) => b.tile));
    const deposits = this.map.deposits;
    for (let i = 0; i < owners.length; i++) {
      if (owners[i] === this.me && deposits[i] !== 0 && !built.has(i)) {
        freeDeposits[DEPOSITS[deposits[i]]] += 1;
      }
    }
    return { freeDeposits };
  }

  /** Premier objectif non atteint de la liste, dans l'ordre. */
  private nextObjective(snap: Snapshot, me: PlayerView, survey: Survey, insight: Insight | null): Objective | null {
    const owned = (kind: BuildingKind) => snap.buildings.some((b) => b.owner === this.me && b.kind === kind);
    const steps: { id: string; reached: boolean; make: () => Objective }[] = [
      {
        id: 'expand',
        reached: me.tiles >= EXPAND_GOAL,
        make: () => ({
          id: 'expand',
          title: 'Étends-toi sur les terres libres',
          why: 'Clique sur une terre libre à côté de toi : une partie de tes soldats part la prendre. Plus de terres = plus de travail et de place pour ta population.',
          progress: [me.tiles, EXPAND_GOAL],
        }),
      },
      {
        id: 'sawmill',
        reached: owned('sawmill'),
        make: () =>
          survey.freeDeposits.grove > 0
            ? {
                id: 'sawmill',
                title: 'Construis une scierie sur un bosquet ▲',
                why: 'Tu as un bosquet dans ton territoire. Une scierie dessus produit beaucoup plus de bois. Clique sur « Scierie » en bas, puis sur le ▲.',
                highlight: 'sawmill',
                deposit: 'grove',
              }
            : {
                id: 'sawmill',
                title: 'Étends-toi jusqu’à un bosquet ▲',
                why: 'Les bosquets (triangles verts) donnent beaucoup de bois avec une scierie. Agrandis ton territoire pour en englober un.',
                deposit: 'grove',
              },
      },
      {
        id: 'farm',
        reached: owned('farm'),
        make: () =>
          survey.freeDeposits.fertile > 0
            ? {
                id: 'farm',
                title: 'Construis une ferme sur des terres fertiles ◆',
                why: 'Une ferme nourrit beaucoup plus de monde : ta population grandira plus vite. Clique sur « Ferme », puis sur le ◆.',
                highlight: 'farm',
                deposit: 'fertile',
              }
            : {
                id: 'farm',
                title: 'Étends-toi jusqu’à des terres fertiles ◆',
                why: 'Les terres fertiles (losanges jaunes, souvent près des rivières) rendent les fermes très productives.',
                deposit: 'fertile',
              },
      },
      {
        id: 'city',
        reached: me.buildings[BUILDINGS.indexOf('city')] > 0 || owned('city'),
        make: () => ({
          id: 'city',
          title: 'Fonde une ville',
          why: 'Une ville ajoute beaucoup de place pour ta population (logement). Place-la à l’intérieur de ton territoire, loin des frontières.',
          highlight: 'city',
        }),
      },
      {
        id: 'academy',
        reached: owned('academy'),
        make: () => ({
          id: 'academy',
          title: 'Construis une Académie',
          why: 'L’Académie permet de lancer des recherches qui améliorent ton économie et ton armée. Une seule par civilisation.',
          highlight: 'academy',
        }),
      },
      {
        id: 'research',
        reached: me.techs.some((t) => t > 0) || me.research !== null,
        make: () => ({
          id: 'research',
          title: 'Lance ta première recherche',
          why: me.buildings[BUILDINGS.indexOf('academy')] > 0
            ? 'Clique sur « ✦ Académie » en haut et choisis une recherche. L’agriculture, par exemple, augmente ta nourriture.'
            : 'Attends la fin du chantier de l’Académie, puis ouvre-la depuis le bouton « ✦ Académie » en haut.',
          highlight: 'academy-open',
        }),
      },
      {
        id: 'quarry',
        reached: owned('quarry'),
        make: () =>
          survey.freeDeposits.quarry > 0
            ? {
                id: 'quarry',
                title: 'Construis une carrière sur un gisement de pierre ■',
                why: 'La pierre ouvre l’archerie, l’écurie, le fort et la mine, et les recherches de niveau 2. Clique sur « Carrière », puis sur le ■.',
                highlight: 'quarry',
                deposit: 'quarry',
              }
            : {
                id: 'quarry',
                title: 'Étends-toi jusqu’à un gisement de pierre ■',
                why: 'Les gisements de pierre (carrés gris, près des montagnes) permettent de construire une carrière.',
                deposit: 'quarry',
              },
      },
      {
        id: 'population',
        reached: me.population >= POPULATION_GOAL,
        make: () => ({
          id: 'population',
          title: `Atteins ${formatNumber(POPULATION_GOAL)} habitants`,
          why: 'Ta population grandit tant que la nourriture est en excédent et qu’il reste de la place. Étends-toi, construis des fermes.',
          progress: [me.population, POPULATION_GOAL],
        }),
      },
      {
        id: 'army',
        reached: me.plan.soldiers_permille >= 300 || this.done.has('army'),
        make: () => ({
          id: 'army',
          title: 'Prépare ton armée : passe les soldats à 30 %',
          why: 'Tes voisins finiront par attaquer. Monte le curseur « Soldats » à gauche : plus de soldats, mais moins de travailleurs. À toi de trouver l’équilibre.',
          highlight: 'soldiers',
        }),
      },
    ];

    for (const step of steps) {
      if (step.reached) this.done.add(step.id);
    }
    const next = steps.find((s) => !this.done.has(s.id));
    if (next) return next.make();

    // Après la liste : conseil stratégique général.
    const land = snap.players.reduce((a, p) => a + p.tiles, 0);
    if (land < snap.land_tiles * 0.9) {
      return {
        id: 'free-play',
        title: 'Développe-toi et surveille tes voisins',
        why: 'Il reste des terres libres : prends-les avant les autres. Construis sur tes gisements, et attaque un voisin quand ta force dépasse nettement la sienne.',
      };
    }
    return this.opportunity(snap, insight);
  }

  /**
   * Milieu de partie : plus de consigne, une opportunité lue dans l'état.
   * Les estimations viennent du moteur (mêmes formules que le combat).
   */
  private opportunity(snap: Snapshot, insight: Insight | null): Objective {
    const name = (id: number) => snap.players.find((p) => p.id === id)?.name ?? '?';
    const rivals = (insight?.estimates ?? []).filter((e) => e.target !== 0);
    const best = [...rivals].sort((a, b) => Number(b.conquers_all) - Number(a.conquers_all) || b.share_permille - a.share_permille)[0];
    if (best && (best.conquers_all || best.share_permille >= 250)) {
      const share = best.conquers_all ? 'tout son territoire' : `≈ ${Math.round(best.share_permille / 10)} % de ses terres`;
      return {
        id: `prey-${best.target}`,
        title: `${name(best.target)} est à ta portée`,
        why: `Ton envoi (⚔ ${formatNumber(best.power)}) contre sa défense (🛡 ${formatNumber(best.defense)}) prendrait ${share}. Survole son territoire pour le détail, clique pour attaquer.`,
      };
    }
    if (rivals.length) {
      return {
        id: 'outmatched',
        title: 'Aucun voisin à ta portée',
        why: 'Avec ton envoi actuel, aucune attaque ne mordrait vraiment (survole un voisin pour voir). Renforce-toi : plus de soldats, envoi plus grand, recherches militaires. Ou attends qu’ils s’usent entre eux.',
        highlight: 'soldiers',
      };
    }
    return {
      id: 'overseas',
      title: 'Tes rivaux sont au-delà de la mer',
      why: 'Aucun voisin par la terre : clique sur une côte ennemie pour y envoyer un bateau. Contrôle 80 % des terres pour gagner.',
    };
  }

  /** Alertes, de la plus urgente à la moins urgente. */
  private alerts(snap: Snapshot, me: PlayerView): HelpTip[] {
    const alerts: HelpTip[] = [];
    const foodRate = me.production.food - me.food_consumption;
    if (foodRate < 0 && me.resources.food < -foodRate * 30) {
      alerts.push({
        urgent: true,
        title: 'Famine en approche',
        text: `Ta nourriture baisse (${formatNumber(foodRate)}/s). Baisse la part de soldats, construis une ferme ou étends-toi vers des plaines.`,
      });
    }
    const attackers = snap.attacks.filter((a) => a.target === this.me);
    if (attackers.length) {
      const names = attackers.map((a) => snap.players.find((p) => p.id === a.attacker)?.name ?? '?').join(', ');
      alerts.push({
        urgent: true,
        title: `${names} t’attaque`,
        text: 'Clique sur son territoire pour contre-attaquer (les armées se percutent), ou monte tes soldats. Un fort ralentit fortement l’ennemi.',
      });
    }
    if (me.population >= me.housing * 0.97) {
      alerts.push({
        title: 'Population au maximum',
        text: 'Plus de place pour grandir : fonde une ville ou étends ton territoire.',
      });
    }
    const full = (['food', 'wood', 'stone', 'iron'] as const).filter((k) => me.resources[k] >= me.storage * 0.98 && me.storage > 0);
    if (full.length) {
      alerts.push({
        title: 'Réserves pleines',
        text: 'Une ressource est au maximum et sa production se perd : dépense-la (bâtiments, soldats) ou fais grandir ta population.',
      });
    }
    const soldiers = me.soldiers.reduce((a, b) => a + b, 0);
    const busy = snap.attacks.some((a) => a.attacker === this.me);
    const land = snap.players.reduce((a, p) => a + p.tiles, 0);
    if (!busy && soldiers >= 60 && land < snap.land_tiles * 0.9) {
      alerts.push({
        title: 'Soldats inoccupés',
        text: `${formatNumber(soldiers)} soldats attendent à la maison. Clique sur des terres libres pour t’étendre.`,
      });
    }
    const affordable = BUILDINGS.filter((_, i) => canPay(me.resources, me.build_costs[i]));
    if (affordable.length && me.resources.wood > 1500) {
      alerts.push({
        title: 'Ressources disponibles',
        text: 'Tu as de quoi construire : regarde la barre du bas (les coûts en rouge sont trop chers).',
      });
    }
    return alerts;
  }
}
