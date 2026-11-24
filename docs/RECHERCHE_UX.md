# FRONTIER — recherche et analyse (UX, design)

22/11/2026. Document de réflexion : **rien n'est décidé ici**, c'est la base pour
choisir ensemble la suite. Aucun code n'a été modifié pour ce document.

## 1. Ce que j'ai mal fait

- J'ai pris des pistes proposées « pour donner l'idée » (âges, métiers, types de
  soldats, garnison) et je les ai **toutes codées d'un coup**, sans les valider avec toi
  ni regarder comment d'autres jeux les rendent compréhensibles.
- J'ai mesuré le jeu avec des **simulations entre IA** (durée, équilibre), mais une IA ne
  se perd pas dans une interface. Je n'ai jamais vérifié la question essentielle :
  **un humain qui lance le jeu comprend-il ce qu'il doit faire ?** La réponse est non.

## 2. Diagnostic de la version actuelle

| Ce que tu ressens | Cause concrète |
|---|---|
| « Les troupes sur le territoire ne font que monter » | Le nombre affiché sous ton nom additionne soldats à la maison **+ soldats partis en attaque + garnison**. Envoyer une attaque ne le fait donc pas baisser, et il grimpe avec la population. Ce n'est pas un calcul faux, mais c'est **trompeur** : ce n'est pas la force défensive réelle. |
| « Ça bouge, on ne sait pas ce que c'est » | Une quinzaine de nombres changent 10 fois par seconde (ressources, production, population, défense, formation…), **sans explication ni hiérarchie**. Rien ne dit lesquels comptent. |
| « On ne sait pas quoi faire » | Aucun objectif, aucune étape guidée. Le joueur arrive devant 9 bâtiments, 4 ressources, 3 âges, 8 curseurs et une carte sans savoir par où commencer. |
| « Plein de fonctionnalités, mais pas expliquées » | Tout est disponible dès le départ dans l'interface (même verrouillé). Pas d'info-bulles qui expliquent *pourquoi* (ex. : pourquoi ma nourriture baisse ? que fait la garnison ?). |
| « Pas design, UX pas bonne » | Panneaux denses de texte en chasse fixe, peu d'icônes, pas de retour visuel quand une action réussit ou échoue (sauf un message qui disparaît), pas de distinction claire entre *état* et *action possible*. |
| Âges | Ajoutés sans que tu les veuilles vraiment. Ils ajoutent une couche (et 2 ressources de plus) avant même que le cœur du jeu soit clair. |

En résumé : **le moteur est riche, mais le jeu n'est pas lisible.** La profondeur
existe ; ce qui manque, c'est la façon de la présenter et de l'apprendre.

## 3. Ce que font les autres

### OpenFront (référence directe)
Sources : guides communautaires ([guide débutant](https://openfrontpro.pages.dev/beginners-guide),
[mécaniques](https://openfrontpro.pages.dev/mechanics)), code source lu précédemment.
- **Une seule ressource** (l'or) et **un seul nombre militaire** (les troupes).
- **Deux curseurs seulement** : part des troupes envoyées par attaque, et répartition
  travailleurs / troupes (les travailleurs font l'or, les troupes défendent et attaquent).
- Un **modèle mental très simple** : « terres libres d'abord, puis voisins faibles ».
- Le terrain se comprend en 3 mots : plaine rapide, colline normale, montagne lente.
- Bâtiments peu nombreux, chacun avec **un rôle évident** : ville (+ population),
  port (revenu), poste de défense (frontière). Coût affiché qui double.
- Construction et diplomatie dans **un menu radial** au clic droit.
- Faiblesse : le jeu lui-même explique peu ; des sites de fans font le travail
  (guides, « 15 premières minutes »).

### Northgard (économie à métiers)
Sources : [revue PC Gamer](https://www.pcgamer.com/uk/northgard-review/),
[guide bonheur](https://www.gamepressure.com/northgard-game-guide/happiness/z8ab0e).
- Les villageois commencent **tous sur la nourriture par défaut** ; on les affecte à un
  métier en construisant le bâtiment correspondant (un bâtiment = un métier visible).
- La nourriture fait croître la population ; le **bonheur** bloque la croissance s'il est
  négatif. Une contrainte saisonnière (l'hiver) donne un rythme.
- Critique récurrente : gérer les villageois devient vite **fastidieux** (micro-gestion).
  À retenir pour FRONTIER : des métiers, oui, mais **sans curseurs partout**.

### Supremacy 1914 / Conflict of Nations (stratégie par provinces)
Sources : [page Steam](https://store.steampowered.com/app/979920),
[Gamepressure](https://gamepressure.com/games/supremacy-1914/z42f57).
- Les ressources viennent **de provinces précises** (la carte a une valeur économique),
  ce qui rejoint ton envie de « carte avec de vraies ressources ».
- Le jeu repose beaucoup sur l'**alliance et la diplomatie** entre joueurs.
- Rythme lent assumé (parties longues), à l'opposé d'OpenFront.

### Principes d'interface pour les jeux de stratégie
Sources : [Game Developer — UI strategy dos and don'ts](https://www.gamedeveloper.com/design/ui-strategy-game-design-dos-and-don-ts),
[erreurs de HUD](https://bootcamp.uxdesign.cc/7-obvious-beginner-mistakes-with-your-games-hud-from-a-ui-ux-art-director-d852e255184a),
[Stellaris](https://www.pcgamer.com/uk/why-i-love-stellaris-slick-interface).
- **Hiérarchiser** : classer chaque information par importance ; le haut de la pile
  visible en permanence, le reste à la demande (info-bulles, panneaux repliables).
- **Un panneau d'information central** plutôt que des infos éparpillées.
- **Signaler les événements importants** (attaque reçue, bâtiment fini, famine) avec un
  signal visuel clair, pas seulement un texte qui disparaît.
- **Chaque raccourci a un bouton visible**, et le bouton affiche son raccourci.
- **La couleur porte l'état** (ex. Stellaris : point blanc = non exploité, vert = exploité).
- Un jeu de stratégie ne doit pas être *pauvre* en information non plus : le but est
  l'ordre, pas la suppression.

### Apprendre le jeu (onboarding)
Sources : [Crusader Kings III — refonte du tutoriel](https://gamedeveloper.com/design/deep-dive-refreshing-the-crusader-kings-iii-tutorial-mode-through-optimized-ux),
[bonnes pratiques](https://inworld.ai/blog/game-ux-best-practices-for-video-game-tutorial-design),
[Apple WWDC 2017](https://developer.apple.com/videos/play/wwdc2017/811).
- CK3 avait un tutoriel de **67 boîtes de texte** ; le finir ne prédisait pas que les
  joueurs restent. Ils l'ont réduit à un tiers et ont **révélé l'interface
  progressivement**, en n'expliquant que ce qui sert tout de suite.
- Principe commun : **une mécanique à la fois, au moment où elle devient utile**
  (« divulgation progressive »), info-bulles juste à temps, tutoriel passable.
- **Expliquer la complexité plutôt que la supprimer** (CK3) : on peut garder un jeu
  riche s'il se dévoile par étapes.

## 4. Ce que j'en retiens pour FRONTIER

1. **Un cœur ultra lisible d'abord** (comme OpenFront) : un nombre de troupes qui veut
   dire quelque chose, une action principale (cliquer pour s'étendre/attaquer), un
   objectif clair.
2. **La profondeur se débloque en jouant** : on ne montre pas 9 bâtiments et 4
   ressources à la première minute ; on les fait apparaître quand ils deviennent utiles.
3. **Chaque nombre affiché répond à une question du joueur** (« suis-je en danger ? »,
   « puis-je construire ? », « pourquoi je n'ai plus de nourriture ? »), et une info-bulle
   l'explique.
4. **Des objectifs guidés** au début (« prends 500 cases », « construis une ferme sur les
   terres fertiles »), qui disparaissent ensuite.
5. **La carte porte l'information** : gisements, bâtiments, menaces visibles directement.
6. **Moins de curseurs** : les métiers s'organisent tout seuls par défaut (comme Northgard
   au départ), le réglage fin reste optionnel.

## 5. Pistes possibles (à choisir ensemble)

**A. Recentrer sur un « OpenFront amélioré »** : troupes / travailleurs, 2 ou 3 ressources
simples liées à la carte, peu de bâtiments à rôle évident, pas d'âges. Le plus lisible, le
plus rapide à rendre agréable. La profondeur viendrait plus tard, par petites touches.

**B. Garder la profondeur actuelle mais la dévoiler progressivement** : mêmes systèmes
(métiers, unités, garnison…), mais interface refaite, tutoriel par objectifs, éléments
qui apparaissent au fil de la partie. Plus long, et le risque de complexité reste.

**C. Entre les deux** : base A, plus une ou deux idées fortes qui font l'identité de
FRONTIER (par exemple les gisements à contrôler, et la garnison qui s'entraîne), sans âges.

Dans tous les cas : refaire l'interface en partant de « que doit savoir le joueur à cet
instant ? », corriger l'affichage des troupes, et **tester avec un humain** (toi) à chaque
étape plutôt que tout livrer d'un bloc.

## 6. Tes décisions (22/11/2026)

- **On garde tous les systèmes**, mais on les **dévoile petit à petit** (ex. : pas de pierre
  au début, donc pas de pierre dans le HUD), avec une aide en jeu (**touche H**) et une
  explication du HUD dans le menu.
- **Les âges sont supprimés**, à remplacer par un meilleur système (voir 7).
- **Apprentissage** : un tutoriel accessible depuis le menu, puis de **petites
  info-bulles** qui disent simplement ce qu'est chaque donnée.
- **Méthode** : petites étapes, chacune testée et validée par toi avant la suivante.

## 7. Par quoi remplacer les âges ? (pistes trouvées)

| Piste | Exemple | Principe | Pour FRONTIER |
|---|---|---|---|
| **Déblocage par les bâtiments** | Les Settlers, Northgard | On débloque une chose en construisant ce qui la produit : une carrière sur un gisement de pierre fait apparaître la pierre et les mineurs ; une forge débloque le fer. | Très lisible, colle à la carte (les gisements comptent), et s'accorde avec « on ne montre la pierre que quand on en a ». |
| **Points de savoir** | Northgard ([PC Gamer](https://www.pcgamer.com/au/northgard-is-like-a-viking-themed-age-of-empires-with-dynamic-seasons)) | Des travailleurs dans un lieu précis (cercle de pierres) produisent du savoir, dépensé dans un petit arbre d'améliorations. | Ajoute une ressource et un arbre : plus de profondeur, plus de choses à expliquer. |
| **Choix à des paliers** | Against the Storm ([wiki](https://wiki.hoodedhorse.com/Against_the_Storm/Unlock)) | À certains paliers (population, territoire…), on choisit **1 amélioration parmi 2 ou 3**. | Donne une identité à chaque partie et renforce les stratégies (guerrier, bâtisseur, tortue) sans menu compliqué. |
| **Lois / spécialisations** | Idée de développeur ([itch.io](https://itch.io/post/324675)) | On adopte des lois avec un avantage et un inconvénient. | Intéressant pour plus tard (diplomatie, gouvernement). |

Recommandation : **déblocage par les bâtiments** pour le contenu (ressources, unités,
métiers), complété par des **choix à des paliers** pour les améliorations. C'est à toi
de trancher.

## 8. Première petite étape proposée : « lisibilité de base »

Sans toucher aux règles du jeu :
1. **Troupes sur la carte** : afficher la force qui défend vraiment le territoire, et
   montrer à part les soldats partis en attaque.
2. **HUD progressif** : n'afficher une ressource que lorsqu'on la produit ou la possède.
3. **Petites info-bulles** sur chaque nombre (une phrase : ce que c'est, d'où ça vient).
4. **Touche H** : un panneau d'aide « que faire maintenant ? » selon la situation.

On teste ensemble, puis on passe à la suppression des âges, puis au tutoriel.

## 9. Ton choix : bâtiments + Académie (façon Rise of Kingdoms)

### Comment fonctionne l'Académie de Rise of Kingdoms
Sources : [wiki RoK — Academy](https://riseofkingdoms.fandom.com/wiki/Buildings/Academy),
[guide AllClash](https://www.allclash.com/rise-of-kingdoms-academy-research-guide-best-path/),
[guide Heaven Guardian](https://heaven-guardian.com/rise-of-kingdoms-academy-guide/).
- Un bâtiment unique, l'Académie, avec **deux branches** : **Économie** (récolte,
  production, vitesse de construction…) et **Militaire** (bonus des troupes, troupes
  de rang supérieur).
- Chaque technologie a **plusieurs niveaux**, des **prérequis**, et coûte **les
  ressources normales** (nourriture, bois, pierre, or) **plus du temps**.
- Une seule recherche à la fois ; certaines technologies accélèrent la recherche ou la
  construction.
- Attention : RoK est un jeu mobile avec des minuteurs de plusieurs heures/jours. Pour
  FRONTIER (parties de 45 min à 1 h), il faut des recherches de **30 s à 3 min**.

### Adaptation proposée pour FRONTIER
**Les bâtiments débloquent le contenu** (travailleurs, ressources, unités) :
- au départ : cueilleurs et bûcherons, nourriture et bois ;
- **carrière** sur un gisement de pierre → la pierre et les mineurs apparaissent ;
- **mine/forge** sur un gisement de fer → le fer et les forgerons apparaissent ;
- **caserne** → forme plus vite ; **archerie** → archers ; **écurie** → cavaliers.
  Plus d'âges : on accède à ces bâtiments dès qu'on peut les payer (la pierre et le fer
  servent naturellement de « marches »).

**L'Académie améliore tout le reste**, par exemple :
- *Économie* : rendement de chaque métier, vitesse des chantiers, logement, réserves ;
- *Militaire* : attaque des guerriers / archers / cavaliers, défense et entraînement de
  la garnison, vitesse d'expansion (plaines, forêts), retraite moins coûteuse ;
- environ 20 technologies à 2–3 niveaux, **impossibles à toutes obtenir** dans une
  partie : chaque joueur se spécialise (guerrier, bâtisseur, tortue).

**Quelle ressource pour la recherche ?** Deux possibilités :
1. *Les ressources existantes* (comme RoK) : rien de nouveau à expliquer, et la recherche
   entre en concurrence avec l'armée et les bâtiments → vrai arbitrage. **Recommandé.**
2. *Une ressource « savoir »* produite par des érudits (5ᵉ métier) : plus de profondeur,
   mais une couche de plus à apprendre.

### Ce que ça implique (avant de coder)
- **Moteur** : retirer les âges (joueur, config, coûts), lier métiers et unités aux
  bâtiments, ajouter un système de **bonus** (multiplicateurs) branché sur la production,
  le combat et l'expansion, un ordre « lancer une recherche », et tout rééquilibrer en
  simulation.
- **IA** : chaque personnalité aura une liste de recherches prioritaires.
- **Interface** : un panneau Académie (arbre lisible, coût, durée, effet en une phrase),
  qui **n'apparaît qu'une fois l'Académie construite** (dévoilement progressif).
- **Multijoueur / ML** : sans problème (c'est un ordre de plus, déterministe).
- **Risque** : encore une interface à rendre claire → à faire après la passe de lisibilité.

### Ordre des petites étapes proposé
1. **Lisibilité de base** (partie 8) — sans changer les règles.
2. **Suppression des âges** → déblocages par les bâtiments.
3. **Académie** (moteur, IA, interface).
4. **Tutoriel** dans le menu + aide H enrichie.

Chaque étape : je code, tu testes, tu valides, puis on passe à la suivante.

## 10. Étape 1 réalisée — à valider (22/11/2026)

Principes appliqués (sources : [université de Waterloo](https://dspacemainprd01.lib.uwaterloo.ca/server/api/core/bitstreams/3efb3218-0ffc-4737-be4a-7c74bc5e9dae/content) —
le joueur garde la main sur l'aide ; [Guidelines for Strategy Game Design](https://www.level99store.com/blogs/guidelines) —
toujours dire quoi faire ; [Kingdoms and Castles](https://gamedeveloper.com/design/teaching-players-without-tutorials-in-i-kingdoms-and-castles-i-) —
apprendre par des objectifs ; [quêtes de Rise of Kingdoms](https://riseofkingdoms.fandom.com/wiki/Quests?oldid=5240) — un seul
objectif actif) et ta consigne : **l'aide accompagne, elle ne dérange pas**.

- **Bande d'objectif** discrète (une ligne + progression) ; l'explication s'affiche
  9 s quand l'objectif change, puis au survol ; masquable. Chaîne : s'étendre (600 cases)
  → scierie sur bosquet → ferme sur terres fertiles → ville → 2 000 habitants →
  soldats à 30 % → âge de la Pierre, puis conseils généraux.
- L'élément concerné **brille doucement** ; les gisements utiles sont mis en évidence.
- **Alertes** : seules les urgences (famine, attaque) s'affichent d'elles-mêmes ; les
  autres conseils (population au max, réserves pleines, soldats inoccupés…) sont dans **H**.
- **Troupes sur la carte** : « ⚔ défense » + « → partis en attaque ».
- **HUD progressif** : ressources, lignes (campagne, garnison), métiers, unités et
  bâtiments n'apparaissent que lorsqu'ils servent ; compteur FPS retiré ; nombres
  rafraîchis ~3 fois par seconde.
- **Petites info-bulles** (une phrase) sur chaque nombre, curseur et bouton.
- Raccourci de l'archerie déplacé sur **G** (H = aide).

## 11. Conception des étapes 2 à 4 (avant de coder)

### Étape 2 — plus d'âges : les bâtiments débloquent
- Tous les bâtiments sont disponibles dès le départ ; ce sont **les coûts** qui créent la
  progression : la carrière ne coûte que du bois ; la pierre qu'elle donne ouvre l'archerie,
  l'écurie, le fort et la mine ; le fer de la mine sert aux recherches avancées.
- **Mineurs** : apparaissent avec la première carrière terminée. **Forgerons** : avec la
  première mine. **Archers** : archerie. **Cavaliers** : écurie.
- La puissance « par âge » disparaît : c'est l'Académie qui renforce les unités.
- Conséquence UX : la chaîne se lit sur la carte (gisements) et dans la barre du bas, pas
  dans un menu d'époques.

### Étape 3 — l'Académie
- Un seul exemplaire par joueur ; coût bois + nourriture (pas de pierre, pour qu'elle soit
  accessible tôt et permette toutes les stratégies).
- **Une recherche à la fois**, payée avec les ressources existantes, 30 s à 2 min.
- 17 technologies à 3 niveaux, réparties en deux branches :
  - **Économie** : agriculture, sylviculture, maçonnerie, métallurgie (rendement de chaque
    métier), architecture (chantiers plus rapides), urbanisme (logement), greniers (réserves) ;
  - **Militaire** : entraînement (formation plus rapide), lames (guerriers), tir (archers),
    équitation (cavaliers), boucliers (défense des soldats), discipline (défense de la
    garnison), instructeurs (entraînement de la garnison), colonisation (expansion),
    logistique (retraite moins coûteuse), fortification (forts).
- Niveau 2 : coûte aussi de la pierre ; niveau 3 : coûte aussi du fer → la carte et les
  gisements comptent jusque dans la recherche.
- Impossible de tout rechercher dans une partie : ~10 à 15 recherches réalistes → on
  choisit sa voie.
- Côté moteur : un **système de bonus** unique (`modifier`) lu par la production, la
  formation, les chantiers, le logement, les réserves, le combat et l'expansion.
- Côté interface : un bouton « Académie » n'apparaît qu'une fois l'Académie construite ;
  panneau en deux colonnes, chaque recherche en une ligne (nom, niveau ●●○, effet en une
  phrase, coût, durée) ; la recherche en cours est visible discrètement en haut.
- Côté IA : une liste de recherches prioritaires par personnalité.

### Étape 4 — apprendre
- **Menu → « Comment jouer »** : quelques cartes courtes (le but ; s'étendre et attaquer ;
  population ; bâtiments et gisements ; Académie ; le HUD expliqué avec un schéma ; commandes).
- **Menu → « S'entraîner »** : une partie sur petite carte contre un adversaire passif.
- En jeu : la bande d'objectif et l'aide H (étape 1) suivent la nouvelle progression
  (carrière, Académie, première recherche).


## 12. Étapes 2 à 4 réalisées (22/11/2026)

- **Âges supprimés.** Les bâtiments débloquent : carrière → mineurs et pierre ; mine →
  forgerons et fer ; archerie → archers ; écurie → cavaliers. Les bâtiments qui coûtent
  de la pierre ou du fer restent cachés tant qu'on n'en a jamais eu.
- **Académie** (une par joueur) : 17 technologies à 3 niveaux, une recherche à la fois ;
  durée ×2 à chaque niveau, niveau 2 = pierre, niveau 3 = fer. Panneau en deux colonnes,
  cadenas qui disent ce qui manque, recherche en cours visible en haut.
- **IA** : chacune a ses priorités de recherche (Colon : économie ; Conquérant : armée ;
  Bastion : garnison et forts ; Opportuniste : un peu de tout).
- **Fin de partie** : un fort pris est détruit (sinon les défenses s'accumulaient et plus
  personne n'avançait) ; la population se relève après une guerre (croissance de base
  liée au logement).
- **Guide** : objectifs s'étendre → scierie → ferme → ville → Académie → première
  recherche → carrière → population → armée.
- **Apprendre** : menu « Comment jouer » (8 cartes courtes, dont un schéma de l'écran),
  ouvert automatiquement à la première visite ; « S'entraîner » (petite carte, un
  adversaire passif).
- **Bug corrigé en testant** : les boutons redessinés plusieurs fois par seconde perdaient
  parfois le clic ; les panneaux ne se redessinent plus que s'ils changent, et les
  boutons réagissent à l'appui.
- Simulations IA : terres prises vers 12-14 min, Académie vers 2-3 min, première
  élimination vers 10-22 min, la moitié des parties finies entre 38 et 54 min.

## 13. Rythme du combat (22/11/2026)

Constat en jouant : attaques trop lentes (5 000 contre 2 500 rampait), expéditions
interminables (3 000 soldats sur des terres libres = plus de 5 min), guerres sans enjeu.

Comparaison avec le code d'OpenFront (`EngineConfig.attackLogic`) :
- chez eux, chaque tour, une attaque prend environ `largeur du front ÷ coût` cases :
  tout le front avance en même temps ; chez nous, le nombre de cases par tour était
  fixe, quelle que soit la largeur (≈ 25 fois plus lent sur un large front) ;
- une case libre leur coûte ≈ 1,6 troupe ; chez nous 0,08 soldat (20 fois moins).

Décisions (volontairement plus lent qu'OpenFront : « 500 ans en 50 min ») :
- **Tout le front avance** : vitesse par case de front (`attack_speed_permille` 120).
  À forces égales ≈ 1,2 case/s (OpenFront ≈ 5), contre un défenseur 2 fois plus
  faible ≈ 2,4 cases/s, au plus 2 fois la vitesse d'égalité (`speed_ratio_min` 500).
- **Terres libres** : 0,4 soldat par case ; une expédition dure 30 à 40 s, puis il
  faut renvoyer du monde (retour de test : 0,7 soldat et 10 s, c'était trop court
  et trop cher).
- **Une terre occupée coûte au moins autant qu'une terre libre** (base 0,5 soldat,
  pertes au moins ×0,8), sinon on avalait un voisin vidé par l'expansion en 2 min.
- **Densité** : l'attaquant paie 40 % de la densité du défenseur ; le défenseur perd
  toute sa densité à chaque case perdue. Une erreur se paie, les guerres tranchent.
- **IA** : prudentes avant 15 min, puis de plus en plus audacieuses ; mobilisation
  quand le logement est plein ; débarquements chez les voisins après 25-30 min.

- **Bug corrigé** : les cases de départ d'une attaque avaient toutes la même
  priorité, donc une expédition courte grandissait toujours vers le nord. Elles ont
  maintenant une priorité bruitée (test `short_expeditions_grow_evenly`).

Simulations (10 graines) : terres prises vers 9-10 min, première élimination vers
9-22 min, parties finies entre 25 et plus de 66 min (forte variance entre IA). Mesures reproductibles :
`cargo test --release -p frontier-engine measure_attack_pace -- --ignored --nocapture`.

## 14. HUD et milieu de partie : conception (22/11/2026)

### Le problème, vu en jouant
Après ~13 min (plus de terres libres), le joueur ne sait plus quoi faire : il ignore
s'il est plus fort que ses voisins, ce que rapportent ses bâtiments, ce que changent
ses curseurs ; les attaques en cours (les siennes, celles qu'il subit, les bateaux)
sont mal visibles. Les textes existants sont vagues et longs.

### Principe directeur : montrer la conséquence, pas expliquer la règle
Ce que font les bons jeux de gestion (Anno, Stellaris/Victoria, Northgard, Frostpunk) :
le joueur comprend parce qu'il **voit l'effet de chaque choix avant et après**, avec
des chiffres, des icônes et des couleurs. Pas de paragraphes, pas de tutoriel
permanent : l'information apparaît **à l'endroit où l'on regarde, au moment où l'on
décide**. Règles retenues :
1. **Aperçu avant d'agir** : survoler un territoire = estimation de l'attaque ;
   bouger un curseur = nouvelles valeurs en fantôme ; survoler un bâtiment à
   construire = ce qu'il rapporte vraiment (bonus de l'Académie compris).
2. **Comparer plutôt que décrire** : ta force contre la sienne, avec un verdict
   coloré (favorable / serré / perdu d'avance).
3. **Le vrai chiffre** : solde réel, détail au survol (bâtiments, terrain, mangé,
   formation) ; jamais « + population maximale » sans le nombre.
4. **Le gaspillage se voit** : réserve pleine, logement plein → chiffre orange.
5. **Une indication seulement où il le faut** : la bande d'objectif reste, et en
   milieu de partie elle devient une **opportunité** calculée (« Kethra est ta
   voisine la plus faible ») plutôt qu'une consigne.

### Ce qui change
- **Moteur** (`insight.rs`, sans effet sur la partie) : issue d'un plan de population
  (travailleurs, production, défense) pour chaque position du curseur ; estimation
  d'une attaque (cases prises, part du territoire, percée) contre chaque cible ;
  production des bâtiments séparée de celle du terrain ; bonus de l'Académie.
- **Curseurs** : pendant qu'on règle Soldats ou Garnison, une ligne fantôme montre
  ◆ ▲ et 🛡 avant → après.
- **Survol d'un territoire** : petite carte « ⚔ ton envoi contre 🛡 sa défense →
  ≈ N cases (X % de son territoire) », verdict coloré ; sur les terres libres, ≈ N cases.
- **Étiquettes sur la carte** : 🛡 force de la garnison · ⚔ soldats à la maison
  (les troupes parties ne sont plus affichées).
- **Panneau des attaques** : ↗ tes attaques, ↘ celles qui te visent, ⛵ bateaux
  (les tiens et ceux qui arrivent) ; clic = centrer la carte.
- **Bâtiments** : info-bulle sur 3 lignes centrées (nom / effet chiffré / coût en
  icônes) ; survol d'un bâtiment posé = sa production réelle.
- **Classement** : couleur, nom, IA ou joueur, % des terres, troupes ; bouton pour
  le masquer ; ⚙ pour ajouter des colonnes (ressources, bâtiments) ; taille réglable.
- **Milieu de partie** : opportunités dans la bande d'objectif, gisements des
  voisins visibles comme des cibles (survol = ce qu'ils rapportent).

### Réalisé (22/11/2026)
Tout ce qui précède est en place. Vérifié : 37 tests Rust (dont l'estimation d'attaque
comparée à de vrais combats, à ±40 %), typage du client, essai dans le navigateur
(aperçu des curseurs, info-bulles des bâtiments, classement réglable, panneau des
attaques, estimation sur les terres libres). Pas encore vu en jouant : l'estimation
contre un voisin et l'objectif « opportunité » de milieu de partie.

## 15. Une société qui a de la mémoire (22/11/2026)

Constat : la population était un jeu de vases communicants sans mémoire (on passe
de 10 % à 60 % de soldats et retour sans conséquence durable), le fer ne servait
pas à l'armée, et une conquête rapportait tout de suite (effet boule de neige).
Cap du propriétaire : « 500 ans en 50 min », chaque choix pèse plus tard.

Mécaniques ajoutées (chiffres dans `GameConfig`) :
- **Expérience des métiers** (`experience_bonus_permille` 500, `experience_time`
  3000) : les travailleurs restés en poste gagnent jusqu'à +50 % de rendement en
  ~5 min. Les derniers arrivés partent les premiers (on ne perd l'expérience qu'en
  vidant vraiment un métier). Réorganiser son économie a un coût durable.
- **Équipement** (`equipment_iron_milli` 400, `equipment_bonus_permille` 500) :
  chaque recrue formée avec 0,4 fer est armée ; une armée entièrement équipée est
  50 % plus forte en attaque et en défense. Le fer devient la ressource militaire,
  les mines deviennent des cibles du milieu de partie.
- **Intégration des conquêtes** (`integration_ticks` 2400 = 4 min) : une case prise
  à un joueur ne loge personne et n'offre pas d'emplois pendant 4 min. Le vaincu
  perd tout de suite, le vainqueur gagne plus tard : frein à la boule de neige.
- **Garnison** plafonnée à ×2,5 (au lieu de ×3) : elle bloquait les fins de partie.
- **Bug corrigé** : la garnison ne s'entraînait jamais tant que la population
  grandissait (arrondi à la baisse à chaque recrue).

Interface : préréglages de société (Économie, Équilibre, Guerre, Tortue) ; pastilles
🗡 équipement et ⏳ cases à intégrer ; expérience et délais (« épuisé dans 1:20 »)
dans les info-bulles des ressources ; délai rouge à côté d'une ressource qui
s'épuise en moins de 3 min ; l'info-bulle des curseurs ne cache plus l'aperçu.

Simulations (10 graines) : terres prises vers 9-10 min, première élimination vers
8-15 min, 9 parties sur 10 finies entre 33 et 65 min (les tirages très courts,
vers 21-25 min, ont disparu).

Écartés pour l'instant : entretien permanent de l'armée (les soldats mangent déjà
1,5 fois plus), marché, saisons — à reconsidérer après des parties humaines.
