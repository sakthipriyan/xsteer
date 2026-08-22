//! WASM surface for the browser.
//!
//! Xsteer ships **one** WASM module rather than loading `xfina-wasm` alongside its own:
//! parsing and planning then share a single `xfina` version, so the parse schema can
//! never drift out of step with the code that consumes it.

use wasm_bindgen::prelude::*;
use xsteer_core::Vault;

/// The `xfina` version this build parses with. Surfaced in the UI so a user reporting
/// a parse problem can say which parser produced it.
#[wasm_bindgen]
pub fn xfina_version() -> String {
    // Kept in step with the `xfina` dependency in Cargo.toml.
    "0.2".to_string()
}

#[wasm_bindgen]
pub fn xsteer_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Round-trip a vault through the engine's own model.
///
/// Placeholder for the real surface (`ingest`, `retag`, `plan`) while those phases are
/// unimplemented — but it does genuinely validate that a vault serialized by the web
/// layer deserializes into the Rust model, which is the contract most likely to break
/// as the model evolves.
#[wasm_bindgen]
pub fn validate_vault(json: &str) -> Result<String, JsError> {
    let vault: Vault = serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
    serde_json::to_string(&vault).map_err(|e| JsError::new(&e.to_string()))
}
