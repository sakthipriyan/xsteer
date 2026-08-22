//! # Xsteer core
//!
//! `xfina` answers *"what happened"*. Xsteer answers *"what should I do this month"*.
//!
//! Everything financial lives here — the web layer decrypts the vault, hands it to
//! this crate, and renders what comes back. See `docs/DESIGN.md`.

pub mod allocator;
pub mod ingest;
pub mod model;
pub mod planner;
pub mod tagging;

use model::*;
use serde::{Deserialize, Serialize};

/// The complete user state. This is what gets encrypted into IndexedDB, and the only
/// thing that crosses the JS/WASM boundary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vault {
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub cards: Vec<CardState>,
    #[serde(default)]
    pub inflows: Vec<Inflow>,
    #[serde(default)]
    pub txns: Vec<Txn>,
    #[serde(default)]
    pub rules: Vec<tagging::Rule>,
    /// Manual per-transaction categorization. Always beats `rules`, so re-running
    /// rules after editing them never clobbers a hand correction.
    #[serde(default)]
    pub overrides: Vec<tagging::Override>,
}

impl Vault {
    pub fn account(&self, id: &AccountId) -> Option<&Account> {
        self.accounts.iter().find(|a| &a.id == id)
    }

    pub fn card(&self, id: &AccountId) -> Option<&CardState> {
        self.cards.iter().find(|c| &c.id == id)
    }

    /// Deposit accounts only — the ones a plan can actually move money between.
    pub fn deposit_accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.iter().filter(|a| a.kind.is_deposit())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("parse failed: {0}")]
    Parse(String),
    #[error("unknown account: {0}")]
    UnknownAccount(AccountId),
    #[error("malformed vault: {0}")]
    Vault(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
