# Architecture de FRONTIER

Ce document explique les choix qui doivent survivre aux itérations. La première
version d'un jeu finit souvent par être la dernière : les fondations ci-dessous
sont faites pour accueillir le multijoueur, l'économie, la diplomatie et des IA
entraînées sans réécriture.

## 1. Les trois principes

### Le moteur est déterministe
Même carte + même configuration + mêmes ordres ⇒ exactement le même état, sur
toutes les machines (navigateur en WebAssembly comme serveur natif).

Conséquences, à respecter dans `engine` (et dans tout ce qui influence l'état) :
- **aucun flottant** dans la simulation : uniquement des entiers (les pour-mille
  remplacent les pourcentages) ;
- **pas d'itération sur `HashMap` / `HashSet`** (ordre aléatoire) : `Vec`,
  `BTreeMap`, tri explicite ;
- **un seul aléatoire** : `engine::rng::Rng`, avec une graine ;
- pas d'horloge, pas d'entrée/sortie ;
- `worldgen` peut utiliser des flottants mais uniquement via la crate `libm`
  (résultats identiques en natif et en WebAssembly) ;
- version de Rust figée dans `rust-toolchain.toml`.

`Game::state_hash()` donne l'empreinte de l'état : le multijoueur s'en servira
pour détecter une désynchronisation, les tests s'en servent déjà.

### Les ordres sont la seule façon d'agir
Un humain, une IA, un joueur distant : tous envoient des `Intent`
(`engine/src/intent.rs`). Le moteur les applique au début du tour suivant, dans
l'ordre reçu. Une IA ne peut donc ni tricher ni faire plus qu'un humain, et le
réseau n'a besoin de transporter que des ordres.

### Les règles sont des données
Tous les chiffres d'équilibrage sont dans `GameConfig` (`engine/src/config.rs`).
La configuration fait partie de la définition d'une partie : elle voyagera avec
la carte en multijoueur. L'exemple `tune` permet de comparer des réglages en
simulation avant de toucher au jeu.

## 2. Les modules

```
             ┌─────────────┐
             │  worldgen   │  graine → MapData (terrain, relief, rivières)
             └──────┬──────┘
                    ▼
┌──────────┐   ┌─────────────┐   ┌────────────┐
│    ai    │──▶│   engine    │◀──│   Intent   │◀── humain (clic) / réseau
│ (Agent)  │   │ Game::tick  │   └────────────┘
└──────────┘   └──────┬──────┘
      ▲               ▼
      │        ┌─────────────┐
      └────────│   session   │  salon → carte + Game + IA, file d'ordres
               └──────┬──────┘
                      ▼
          wasm (navigateur) · serveur (à venir) · Python (à venir)
```

| Crate | Rôle | Dépend de |
|---|---|---|
| `engine` | état de la partie, règles, tour de jeu | — |
| `worldgen` | génération procédurale, contrôles de qualité | engine |
| `ai` | trait `Agent`, perception, IA de base, `BotHost` | engine |
| `session` | assemble une partie depuis la config du salon | engine, worldgen, ai |
| `wasm` | API JavaScript fine au-dessus de `session` | session |

Le moteur ne sait pas qui est humain ou IA, ni comment la carte a été faite :
`MapData` est une donnée. Une carte dessinée à la main pourra être chargée
exactement comme une carte générée.

## 3. Le tour de jeu (10 par seconde)

`Game::tick(ordres)` (`engine/src/game.rs`, chaque système dans son fichier) :
1. applique les ordres (placement, attaque, rappel, bateau, construction, plan de
   population, passage d'âge) — un ordre invalide produit un événement `Rejected` ;
2. phase de placement (`spawn.rs`) ;
3. population (`population.rs`) : production des métiers (emplois du terrain et des
   bâtiments), consommation de nourriture, naissances selon l'excédent, famine,
   formation des soldats et de la garnison au rythme des casernes, répartition des
   travailleurs (automatique ou manuelle) ; avancement de la recherche (`research.rs`) ;
4. chantiers (`buildings.rs`) ;
5. attaques (`attack.rs`) : rapport défense / attaque borné, terrain, rivières, forts,
   pertes réparties entre types d'unités, front qui avance par « réserve d'avancée » ;
6. bateaux (`naval.rs`) ;
7. éliminations, puis victoire.

Personnes et ressources sont comptées en **millièmes** (entiers) : `1000` = une
personne. Les catégories (métiers, unités, gisements, bâtiments, technologies, effets)
sont dans `defs.rs`, leurs chiffres dans `GameConfig` (tables `units`, `buildings`,
`techs`, `jobs_per_tile`...).

**Bonus de l'Académie** : un seul point d'entrée, `Game::modifier(joueur, Effect)`, lu par
la production, la formation, les chantiers, le logement, les réserves, le combat et
l'expansion. Ajouter une technologie = une ligne dans `GameConfig::techs` (et son texte
dans `client/src/ui/labels.ts`), sans toucher aux systèmes.

**Aperçus pour l'interface** (`insight.rs`, lecture seule) : issue d'un plan de
population, estimation d'une attaque (mêmes formules que le combat, testée contre
de vrais combats), production des bâtiments et du terrain. La session les expose
(`Session::insight`), le worker les envoie au client quelques fois par seconde.

Le design et les décisions sont dans `docs/RECHERCHE_UX.md` (et `docs/DESIGN_V03.md`).

## 4. Le client

- `engine/worker.ts` fait tourner le moteur WebAssembly dans un Web Worker et
  cadence les tours. En solo, ce worker joue le rôle du serveur.
- `render/renderer.ts` dessine la carte avec WebGL2 : quelques textures
  (1 texel = 1 case : terrain, éclairage du relief, propriétaires, palette)
  et des shaders. Le coût ne dépend pas de la taille de la carte et le zoom est
  gratuit. Seules les lignes modifiées de la texture des propriétaires sont
  renvoyées au GPU.
- `render/overlay.ts` : noms des civilisations (canvas 2D).
- `ui/` : interface HTML (HUD, info-bulles `data-tip`, aide H, panneau Académie) ;
  `app/` : menu, tutoriel (`tutorial.ts`), écran de jeu et **guide** (`guide.ts` :
  objectif en cours et alertes, calculés à partir de l'état, sans jamais jouer).
- Principe UX : dévoilement progressif — une ressource, une ligne, un bâtiment
  n'apparaît que lorsqu'il devient utile.
- `engine/protocol.ts` reflète les types JSON de Rust : **à mettre à jour
  quand un type serde change côté Rust**.

Direction artistique : palette et éclairage du prototype Python
(`render/palette.ts`), panneaux sombres translucides, police à chasse fixe.

## 5. Ajouter une IA

1. Créer `crates/ai/src/agents/mon_ia.rs` avec une structure qui implémente
   `Agent::act(&mut self, game: &Game, me: PlayerId, out: &mut Vec<Intent>)`.
   `BotCore` fournit le placement, le rythme de réflexion et la difficulté ;
   `sense::sense` résume le voisinage. `passif.rs` est le point de départ le
   plus simple.
2. L'ajouter dans `AGENTS` et `create_agent` (`crates/ai/src/lib.rs`).
3. Elle apparaît dans le menu. La tester contre les autres avec l'exemple
   `simulate`.

Une IA n'a pas besoin d'être déterministe : en multijoueur, les IA tourneront
sur le serveur et enverront leurs ordres comme des joueurs. Une IA en Python ou
un réseau de neurones pourra donc jouer, à condition de passer par des `Intent`.

## 6. Ajouter un système de jeu (économie, villes, diplomatie…)

- L'état va dans `Game` (ou un sous-module de `engine`), les chiffres dans
  `GameConfig`, les actions du joueur dans de **nouvelles** variantes
  d'`Intent`, ce qu'il faut afficher dans de nouveaux `GameEvent` et dans
  `session::Snapshot`.
- Le système s'insère à une place fixe dans `Game::tick`.
- Ajouter un test de déterminisme et un test de cohérence (voir
  `crates/session/tests/bot_games.rs`).
- Ne jamais changer le sens d'une variante existante d'`Intent` ou de
  `Terrain` (les valeurs numériques sont utilisées par le rendu et les IA).

## 7. Ce qui vient ensuite

| Étape | Où | Idée |
|---|---|---|
| ~~Économie, villes, forts, bateaux~~ | engine | fait (v0.2) |
| ~~Métiers, unités, garnison, gisements, Académie~~ | engine | fait (v0.3) |
| Combats navals | engine | intercepter les bateaux, ports |
| Diplomatie | engine | alliances, paix, commerce ; les IA les utilisent via des `Intent` |
| Multijoueur | nouveau crate `server` | salons WebSocket ; le serveur regroupe les ordres par tour et les diffuse, chaque client simule ; vérification par `state_hash` |
| IA en Python / ML | nouveau crate `py` (PyO3) | `env.reset()`, `env.step(ordres)`, observations en tableaux numpy, milliers de parties sans affichage |
| Cartes faites main | worldgen ou fichiers | charger un `MapData` depuis un fichier |
| Sauvegarde / replays | session | une partie = setup + liste d'ordres : il suffit de la rejouer |

Points de vigilance connus : l'équilibrage reste jeune (ajuster avec les exemples
`tune` et `inspect`) ; environ la moitié des parties entre IA dépassent l'heure.
