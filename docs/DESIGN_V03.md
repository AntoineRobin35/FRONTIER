> **Mise à jour (22/11/2026)** : les **âges ont été supprimés**. Les bâtiments débloquent
> métiers, ressources et unités, et l'**Académie** (recherches) remplace la puissance par
> âge. Voir `docs/RECHERCHE_UX.md`, parties 9 à 11. Le reste de ce document reste valable.

# FRONTIER v0.3 — game design

Validé le 22/11/2026 avec le propriétaire du projet. Tous les chiffres vivent dans
`GameConfig` et s'ajustent par simulation (`cargo run --release -p frontier-session --example tune`).

## Intention
- Un vrai jeu de stratégie : une partie dure **45 min à 1 h**.
- **Début lent, peu de boule de neige** : on choisit sa stratégie principale tôt.
- Plusieurs stratégies viables : **guerrier** (grosse armée, expansion), **bâtisseur / monopole**
  (économie, gisements), **tortue** (garnison entraînée, puis contre-attaque). Chacune a ses risques.
- On n'est normalement pas en manque de ressources, mais la carte compte : les gisements
  rendent beaucoup plus et valent qu'on se batte pour eux.
- Progression sans attente absurde : constructions de 15 à 40 s, passage d'âge en 1 à 1 min 30.

Inspirations : Age of Empires (âges), Les Settlers / Northgard (métiers, gisements),
OpenFront (combat territorial par rapport de forces).

## 1. Trois âges
| Âge | Débloque | Prérequis du passage |
|---|---|---|
| **Bois** (départ) | cueilleurs, bûcherons, guerriers, ville, caserne, ferme, scierie | — |
| **Pierre** | mineurs, archers (archerie), cavaliers (écurie), carrière, fort | 1 ville, nourriture + bois, 60 s |
| **Fer** | forgerons, mine de fer ; toutes les unités deviennent bien plus fortes | 2 villes, nourriture + bois + pierre, 90 s |

Chaque âge multiplie la puissance de tous les soldats (équipement).

## 2. Population
- Population totale = travailleurs + soldats + garnison (y compris les soldats partis en campagne).
- Elle grandit si la **nourriture** est en excédent, jusqu'au **logement** (territoire, de façon
  sous-linéaire, + villes). En famine, elle diminue.
- **Mode simple** (par défaut) : un curseur « part de soldats ». Les travailleurs se répartissent
  automatiquement (nourriture d'abord, puis selon les emplois disponibles).
- **Mode détaillé** (bouton dans le HUD) : on règle chaque métier, chaque type de soldat et la garnison.
- Changer la répartition prend du temps : former un soldat coûte des ressources et se fait à un
  rythme limité (plus rapide avec des casernes). Renvoyer des soldats au travail est gratuit.

### Métiers
| Métier | Produit | Où il travaille | Âge |
|---|---|---|---|
| Cueilleur-chasseur | nourriture | plaines, marais, forêts (peu) ; **ferme** sur terres fertiles (beaucoup) | Bois |
| Bûcheron | bois | forêts ; **scierie** sur bosquet | Bois |
| Mineur | pierre | collines, plateaux (peu) ; **carrière** sur gisement de pierre | Pierre |
| Forgeron | fer | uniquement **mine** sur gisement de fer | Fer |

Chaque case et chaque bâtiment offre un nombre d'**emplois**. Les emplois des bâtiments rendent
environ 4 fois plus que ceux du terrain.

### Soldats
| Unité | Attaque | Défense | Particularité | Débloquée par |
|---|---|---|---|---|
| Guerrier (corps à corps) | 1,0 | 1,0 | polyvalent, efficace contre les forts | départ (caserne : formation plus rapide) |
| Archer (distance) | 0,8 | 1,4 | fort en forêt, collines ; excellent défenseur | archerie (âge de Pierre) |
| Cavalier | 1,3 | 0,7 | fort et rapide en terrain ouvert, faible en forêt / marais / collines | écurie (âge de Pierre) |
| **Garnison** (défenseur) | — | 1,0 → 3,0 | ne peut pas attaquer ; s'entraîne avec le temps et devient bien plus fort | départ |

La garnison est la stratégie « tortue » : on ralentit son développement pour devenir très dur à
attaquer, puis on attaque sans craindre la riposte. Si l'économie ne suit pas, c'est perdu.

## 3. Combat (inspiré d'OpenFront)
- Une attaque emporte une part des soldats mobiles (selon leur composition).
- À chaque case, on compare la **défense totale** du défenseur (soldats à la maison + garnison
  entraînée) à la **puissance de l'attaque** :
  - défenseur faible → l'attaque avance vite et perd peu ;
  - défenseur fort → l'attaque ralentit et fond.
- **Terrain** : forêt, marais, collines, plateau et rivières donnent un bonus au défenseur ;
  chaque type d'unité a ses terrains favoris.
- **Fort** : défense ×3 dans un rayon.
- **Retraite** : rappeler une attaque contre un joueur coûte 25 % des troupes.
- **Immunité** de 30 s en début de partie.
- **Terres libres** : peu coûteuses, la vitesse dépend de la taille de l'expédition.

## 4. Carte et gisements
Gisements visibles, répartis équitablement sur toutes les masses terrestres :
terres fertiles (près des rivières), bosquets (en forêt), pierre (collines, près des montagnes),
fer (rare, au pied des montagnes). Les cartes faites à la main restent possibles plus tard.

## 5. Bâtiments
Construction en quelques dizaines de secondes, coût croissant, distances minimales.
Ville, caserne, ferme, scierie (Bois) ; archerie, écurie, carrière, fort (Pierre) ; mine (Fer).
Une barre de construction affiche toujours icône, coût, effet et âge requis.

## 6. Lisibilité
Terrain atténué sous les territoires, couleurs des joueurs plus présentes, frontières nettes,
icônes des gisements et bâtiments, chantiers visibles, HUD : ressources en haut, population à
gauche, construction en bas.

## 7. Plus tard (v0.4+)
Ports et combats navals, diplomatie, recherche d'améliorations, refonte graphique, multijoueur.
