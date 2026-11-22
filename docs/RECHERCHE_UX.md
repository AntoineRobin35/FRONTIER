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
