//! Interface JavaScript du moteur. Volontairement fine : toute la logique
//! reste dans `frontier-session` et les crates en dessous.
//!
//! Les grilles sont renvoyées sous forme de tableaux typés (copies) ; les
//! petites structures en JSON.

use frontier_engine::Intent;
use frontier_session::{Session, SessionSetup};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Liste des IA disponibles (JSON).
#[wasm_bindgen]
pub fn agents_json() -> String {
    serde_json::to_string(frontier_ai::AGENTS).unwrap_or_else(|_| "[]".into())
}

#[wasm_bindgen]
pub struct WasmSession {
    inner: Session,
}

#[wasm_bindgen]
impl WasmSession {
    /// Crée une partie à partir de la configuration du salon (JSON).
    #[wasm_bindgen(constructor)]
    pub fn new(setup_json: &str) -> Result<WasmSession, JsError> {
        let setup: SessionSetup = serde_json::from_str(setup_json)?;
        let inner = Session::new(&setup)?;
        Ok(WasmSession { inner })
    }

    pub fn width(&self) -> u32 {
        self.inner.game().map().width
    }

    pub fn height(&self) -> u32 {
        self.inner.game().map().height
    }

    pub fn tick_ms(&self) -> u32 {
        self.inner.game().config().tick_ms
    }

    pub fn terrain(&self) -> Vec<u8> {
        self.inner.game().map().terrain.iter().map(|&t| t as u8).collect()
    }

    pub fn elevation(&self) -> Vec<u16> {
        self.inner.game().map().elevation.clone()
    }

    pub fn river(&self) -> Vec<u8> {
        self.inner.game().map().river.clone()
    }

    /// Tracés des rivières, à plat : `[nb_tracés, (nb_points, x, y, l, ...)...]`.
    pub fn rivers(&self) -> Vec<f32> {
        let paths = &self.inner.game().map().rivers;
        let mut out = vec![paths.len() as f32];
        for path in paths {
            out.push(path.points.len() as f32);
            for p in &path.points {
                out.extend_from_slice(p);
            }
        }
        out
    }

    /// Gisement de chaque case (`Deposit as u8`).
    pub fn deposits(&self) -> Vec<u8> {
        self.inner.game().map().deposit.clone()
    }

    /// Centres des gisements, à plat : `[case, type, case, type, ...]`.
    pub fn deposit_sites(&self) -> Vec<u32> {
        let mut out = Vec::new();
        for site in &self.inner.game().map().deposit_sites {
            out.push(site.tile);
            out.push(site.kind as u32);
        }
        out
    }

    /// Règles de la partie (JSON) : coûts, effets, âges… pour l'interface.
    pub fn config_json(&self) -> String {
        serde_json::to_string(self.inner.game().config()).unwrap_or_else(|_| "{}".into())
    }

    pub fn owners(&self) -> Vec<u16> {
        self.inner.game().owners().to_vec()
    }

    /// Ordre d'un joueur (JSON), appliqué au prochain tour.
    pub fn submit(&mut self, player: u16, intent_json: &str) -> Result<(), JsError> {
        let intent: Intent = serde_json::from_str(intent_json)?;
        self.inner.submit(player, intent);
        Ok(())
    }

    /// Calcule un tour. Renvoie les changements de propriétaire à plat :
    /// `[case, propriétaire, case, propriétaire, ...]`.
    pub fn step(&mut self) -> Vec<u32> {
        let update = self.inner.step();
        let changed = update.changed_tiles.clone();
        let game = self.inner.game();
        let mut out = Vec::with_capacity(changed.len() * 2);
        for tile in changed {
            out.push(tile);
            out.push(game.owner(tile) as u32);
        }
        out
    }

    /// État résumé du dernier tour (JSON).
    pub fn snapshot_json(&self) -> String {
        serde_json::to_string(&self.inner.snapshot()).unwrap_or_else(|_| "{}".into())
    }

    pub fn state_hash(&self) -> String {
        format!("{:016x}", self.inner.game().state_hash())
    }
}
