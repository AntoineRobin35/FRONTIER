// « Comment jouer » : quelques cartes courtes, une idée par carte.
// Principe (docs/RECHERCHE_UX.md) : peu de texte, ce qui sert tout de suite,
// passable à tout moment. Le reste s'apprend en jouant (objectifs, aide H,
// info-bulles).

interface Card {
  title: string;
  body: string;
}

const CARDS: Card[] = [
  {
    title: 'Le but',
    body: `
      <p>Bâtis la civilisation la plus puissante sur une carte générée.</p>
      <p><b>Victoire :</b> contrôler 80 % des terres, ou être le dernier survivant.</p>
      <p>Une partie dure environ <b>45 minutes à 1 heure</b>. Le début est calme : tu as le temps de choisir ta stratégie.</p>`,
  },
  {
    title: 'S’étendre et attaquer',
    body: `
      <p><b>Clique sur une zone</b> à côté de ton territoire : une partie de tes soldats part la prendre.
        Le curseur <b>Envoi</b> règle combien tu en envoies.</p>
      <ul>
        <li>Les <b>terres libres</b> se prennent facilement.</li>
        <li>Face à un <b>voisin</b>, tout dépend du rapport de forces : un voisin faible tombe vite, un voisin fort ralentit et coûte cher.</li>
        <li>Forêts, collines, marais, rivières et <b>forts</b> aident celui qui défend.</li>
        <li>Une autre île ? Ton clic envoie un <b>bateau</b>.</li>
      </ul>`,
  },
  {
    title: 'Ta population',
    body: `
      <p>Ta population grandit tant que tu produis <b>plus de nourriture</b> que tu n’en manges, jusqu’à ta place maximale (territoire + villes).</p>
      <p>Le curseur <b>Soldats</b> choisit la part qui se bat. Les autres <b>travaillent</b> : nourriture, bois, puis pierre et fer.</p>
      <p>La <b>garnison</b> (réglages détaillés) ne fait que défendre, mais devient de plus en plus forte avec le temps.</p>
      <p>Tes choix durent : un travailleur qui reste à son poste gagne en <b>expérience</b> (jusqu’à +50 %), une recrue formée avec du <b>fer</b> est armée (+50 %), une terre conquise met quelques minutes à s’<b>intégrer</b>.</p>`,
  },
  {
    title: 'Bâtiments et gisements',
    body: `
      <p>La carte cache des <b>gisements</b> qui rendent beaucoup plus avec le bon bâtiment :</p>
      <ul class="legend">
        <li><span style="color:#e3c25a">◆</span> terres fertiles → <b>ferme</b></li>
        <li><span style="color:#4f9a4a">▲</span> bosquet → <b>scierie</b></li>
        <li><span style="color:#c4c5cc">■</span> pierre → <b>carrière</b> (débloque la pierre)</li>
        <li><span style="color:#c27a4a">⬣</span> fer → <b>mine</b> (débloque le fer)</li>
      </ul>
      <p>La <b>ville</b> donne de la place, la <b>caserne</b> forme plus vite, l’<b>archerie</b> et l’<b>écurie</b> débloquent archers et cavaliers, le <b>fort</b> protège une zone.</p>`,
  },
  {
    title: 'L’Académie',
    body: `
      <p>Construis <b>une Académie</b> pour lancer des recherches : rendements, logement, attaque de chaque type de soldat, garnison, expansion…</p>
      <p>Une recherche à la fois. Le <b>niveau 2</b> coûte aussi de la pierre, le <b>niveau 3</b> du fer.</p>
      <p>Tu ne pourras pas tout rechercher : <b>choisis ce qui sert ta stratégie</b>.</p>`,
  },
  {
    title: 'L’écran de jeu',
    body: `
      <div class="hud-map">
        <div class="hm top">Temps · <b>Académie</b> · <b>ressources</b> (stock, +/s) · population · Aide</div>
        <div class="hm obj">🎯 Objectif en cours</div>
        <div class="hm left">Ta population<br>curseurs <b>Soldats</b> et <b>Envoi</b><br>attaques en cours</div>
        <div class="hm right">Civilisations<br>% des terres · ⚔ soldats</div>
        <div class="hm bottom">Construction (coût, raccourci)</div>
        <div class="hm map">Carte : sous chaque nom, 🛡 garnison · ⚔ soldats à la maison. Survole un territoire : ce que donnerait ton attaque.</div>
      </div>
      <p class="small">Survole n’importe quel nombre pour savoir ce que c’est. Les éléments apparaissent au fur et à mesure qu’ils deviennent utiles.</p>`,
  },
  {
    title: 'Commandes',
    body: `
      <table class="keys">
        <tr><td>Clic</td><td>Se placer, s’étendre, attaquer</td></tr>
        <tr><td>Glisser · ZQSD · flèches</td><td>Déplacer la carte</td></tr>
        <tr><td>Molette</td><td>Zoom</td></tr>
        <tr><td>A / E</td><td>Part des soldats envoyés</td></tr>
        <tr><td>Lettre du bâtiment</td><td>Construire (V ville, R ferme, B scierie…)</td></tr>
        <tr><td>Clic droit · Échap</td><td>Annuler</td></tr>
        <tr><td>H</td><td>Aide : que faire maintenant ?</td></tr>
        <tr><td>Espace · 1 2 3</td><td>Pause, vitesse</td></tr>
      </table>`,
  },
  {
    title: 'Trois façons de jouer',
    body: `
      <ul>
        <li><b>Guerrier</b> : beaucoup de soldats, casernes, attaques rapides. Économie fragile.</li>
        <li><b>Bâtisseur</b> : fermes, scieries, villes, recherches économiques. Fort sur la durée, vulnérable au début.</li>
        <li><b>Tortue</b> : garnison entraînée et forts, puis contre-attaque. Risqué si l’économie ne suit pas.</li>
      </ul>
      <p>Pour commencer : <b>S’entraîner</b>, et suis la bande d’objectif en haut de l’écran.</p>`,
  },
];

export class Tutorial {
  readonly el: HTMLElement;
  private index = 0;

  constructor(private onPractice: () => void) {
    this.el = document.createElement('div');
    this.el.className = 'tutorial-backdrop hidden';
    this.el.addEventListener('click', (e) => {
      const target = e.target as HTMLElement;
      if (target === this.el || target.closest('[data-close]')) this.close();
      if (target.closest('[data-prev]')) this.go(this.index - 1);
      if (target.closest('[data-next]')) this.go(this.index + 1);
      if (target.closest('[data-practice]')) {
        this.close();
        this.onPractice();
      }
      const dot = target.closest<HTMLElement>('[data-dot]');
      if (dot) this.go(Number(dot.dataset.dot));
    });
    window.addEventListener('keydown', (e) => {
      if (this.el.classList.contains('hidden')) return;
      if (e.key === 'Escape') this.close();
      if (e.key === 'ArrowRight') this.go(this.index + 1);
      if (e.key === 'ArrowLeft') this.go(this.index - 1);
    });
  }

  open(): void {
    this.index = 0;
    this.render();
    this.el.classList.remove('hidden');
  }

  close(): void {
    this.el.classList.add('hidden');
  }

  private go(index: number): void {
    this.index = Math.max(0, Math.min(CARDS.length - 1, index));
    this.render();
  }

  private render(): void {
    const card = CARDS[this.index];
    const last = this.index === CARDS.length - 1;
    const dots = CARDS.map((_, i) => `<button class="dot ${i === this.index ? 'on' : ''}" data-dot="${i}" aria-label="Carte ${i + 1}"></button>`).join('');
    this.el.innerHTML = `
      <div class="tutorial panel" role="dialog" aria-label="Comment jouer">
        <div class="tuto-head">
          <span class="tuto-step">${this.index + 1} / ${CARDS.length}</span>
          <button class="btn btn-small" data-close>Fermer</button>
        </div>
        <h2>${card.title}</h2>
        <div class="tuto-body">${card.body}</div>
        <div class="tuto-foot">
          <button class="btn" data-prev ${this.index === 0 ? 'disabled' : ''}>← Précédent</button>
          <div class="dots">${dots}</div>
          ${last ? '<button class="btn btn-primary" data-practice>S’ENTRAÎNER</button>' : '<button class="btn" data-next>Suivant →</button>'}
        </div>
      </div>`;
  }
}
