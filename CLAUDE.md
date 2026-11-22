# FRONTIER — consignes pour les sessions de développement

Jeu de stratégie territoriale web. Moteur Rust (crates/) compilé en WebAssembly,
client TypeScript + WebGL2 (client/). Lire `docs/ARCHITECTURE.md` avant tout
changement de structure. Le propriétaire du projet débute en Rust et TypeScript :
expliquer simplement, en français.

## Règles non négociables
- Le moteur (`crates/engine`) est déterministe : entiers uniquement, pas de
  `HashMap`/`HashSet` itérés, aléatoire via `engine::rng::Rng`, pas d'horloge ni d'E/S.
- `crates/worldgen` : flottants autorisés mais fonctions mathématiques via `libm`.
- Toute action de joueur ou d'IA passe par un `Intent`. Ajouter des variantes,
  ne jamais changer le sens d'une existante (idem pour `Terrain`).
- Chiffres d'équilibrage dans `GameConfig`, pas en dur dans les systèmes. Personnes
  et ressources en millièmes (`MILLI`). Vérifier le rythme avec l'exemple `tune`.
- Game design et décisions : `docs/RECHERCHE_UX.md` (puis `docs/DESIGN_V03.md`).
- UX : l'aide accompagne sans déranger (bande d'objectif d'une ligne, alertes urgentes
  seulement, aide sur H) ; tout nombre affiché a une info-bulle (`data-tip`) ; on ne
  montre une chose que lorsqu'elle devient utile.
- Méthode avec le propriétaire : proposer et valider par petites étapes ; ne pas coder
  d'un bloc des idées qu'il présente comme des pistes.
- Types serde modifiés côté Rust ⇒ mettre à jour `client/src/engine/protocol.ts`.
- Garder la direction artistique : palette de `client/src/render/palette.ts`.

## Vérifier un changement
- `cargo test --release --workspace` (depuis `frontier/`)
- `npm run typecheck` (depuis `client/`)
- `cargo run --release -p frontier-session --example simulate -- 42` pour voir une partie IA
- Jeu : `npm run dev` dans `client/` puis http://localhost:5173

Sous Windows, `cargo` est dans `%USERPROFILE%\.cargo\bin` (le script
`client/scripts/build-wasm.mjs` l'ajoute tout seul au PATH).
