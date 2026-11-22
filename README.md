# FRONTIER

Jeu de stratégie territoriale dans le navigateur : des civilisations, une carte
générée, des frontières qui bougent. Tu joues contre des IA (et bientôt contre
d'autres joueurs).

- **Moteur** en Rust (déterministe, rapide, réutilisable pour le multijoueur et
  pour entraîner des IA en Python).
- **Client** en TypeScript + WebGL2 (rendu de la carte par la carte graphique).

Le prototype Python d'origine est conservé dans le dossier parent (`../main.py`, etc.).

## Lancer le jeu

Prérequis (déjà installés sur cette machine) : [Node.js](https://nodejs.org) ≥ 20,
[Rust](https://rustup.rs) avec `wasm-pack` (`cargo install wasm-pack`).

```powershell
cd client
npm install
npm run dev
```

Puis ouvrir <http://localhost:5173>. La première compilation du moteur prend
environ une minute, les suivantes quelques secondes.

## Commandes en jeu

Première fois ? Dans le menu : **Comment jouer**, puis **S'entraîner**. En jeu, suis la
bande 🎯 Objectif en haut, appuie sur **H** pour l'aide, et survole n'importe quel nombre.

| Action | Commande |
|---|---|
| Se placer, s'étendre, attaquer | Clic gauche (bateau si la cible est sur une autre île) |
| Construire | Barre du bas ou lettre (V ville, K caserne, R ferme, B scierie, P carrière, G archerie, U écurie, T fort, M mine, L académie), puis clic sur ton territoire. Maj : en poser plusieurs. Clic droit / Échap : annuler |
| Recherches | Bouton « ✦ Académie » en haut (une fois l'Académie construite) |
| Population | Curseur « Soldats » ; « Réglages détaillés » pour les métiers, l'armée et la garnison |
| Aide | H |
| Déplacer la carte / zoom | Glisser, ZQSD (WASD), flèches / molette |
| Part des soldats envoyés | Curseur « Envoi », ou A / E (Q / E en QWERTY) |
| Pause / vitesse | Espace / 1, 2, 3 |

Règles en bref :
- **Population** : elle grandit si la nourriture est en excédent, jusqu'au logement
  (territoire + villes). Tu choisis la part de **soldats** ; les autres travaillent.
- **Bâtiments et gisements** : la ferme (terres fertiles ◆), la scierie (bosquet ▲), la
  carrière (pierre ■, débloque les mineurs) et la mine (fer ⬣, débloque les forgerons)
  rendent beaucoup plus que le terrain. Archerie et écurie débloquent archers et
  cavaliers ; le fort protège une zone (détruit si l'ennemi le prend).
- **Académie** : une recherche à la fois, 17 technologies à 3 niveaux (économie et
  militaire). Niveau 2 : pierre en plus ; niveau 3 : fer en plus. On ne peut pas tout
  rechercher : on choisit sa voie.
- **Armée** : guerriers, archers, cavaliers, et la **garnison** qui ne fait que défendre
  mais s'entraîne (jusqu'à ×3).
- **Combat** : rapport de forces ; terrain, rivières et forts aident le défenseur ;
  retraite face à un joueur = 25 % de pertes ; trêve de 30 s au début.
- Victoire : 80 % des terres ou dernier survivant. Une partie dure ~45 min à 1 h.

## Commandes de développement

À lancer depuis le dossier `frontier/` :

```powershell
cargo test --release --workspace                                  # tous les tests Rust
cargo run --release -p frontier-session --example simulate -- 42  # partie IA contre IA sans affichage
cargo run --release -p frontier-session --example tune -- 1 2 3   # rythme des parties (âges, éliminations, durée)
cargo run --release -p frontier-session --example inspect -- 1 2  # économie d'une IA minute par minute
```

Depuis `client/` : `npm run typecheck` (vérification TypeScript), `npm run build`
(version de production dans `client/dist`).

## Organisation

```
frontier/
├── crates/
│   ├── engine/     moteur de simulation pur (population, économie, combat, bâtiments)
│   ├── worldgen/   génération procédurale des mondes
│   ├── ai/         interface commune des IA + IA de base
│   ├── session/    assemble une partie (carte + moteur + humains + IA)
│   └── wasm/       pont vers le navigateur
├── client/         jeu web (TypeScript, WebGL2)
└── docs/
    ├── ARCHITECTURE.md   comment tout s'articule, et ce qui vient ensuite
    ├── RECHERCHE_UX.md   recherche, décisions, conception (UX, Académie, tutoriel)
    └── DESIGN_V03.md     game design v0.3 (les âges y sont remplacés : voir RECHERCHE_UX)
```

Pour ajouter une IA, voir `crates/ai/src/lib.rs` et `docs/ARCHITECTURE.md`.
