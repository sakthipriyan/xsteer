//! Categorization: an ordered rule list, plus manual overrides that always win.
//!
//! Phase 3. The types below are referenced by [`Vault`](crate::Vault) and so are
//! defined now; matching itself is not yet implemented.

use crate::model::{CategoryId, TagId, TxnId};
use serde::{Deserialize, Serialize};

/// Rules are evaluated in order; first match wins.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub matcher: Matcher,
    pub category: CategoryId,
    #[serde(default)]
    pub tags: Vec<TagId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Matcher {
    /// Regex against the normalized narration.
    Narration {
        pattern: String,
    },
    AmountBetween {
        min: crate::model::Money,
        max: crate::model::Money,
    },
    Counterparty {
        name: String,
    },
    All {
        of: Vec<Matcher>,
    },
}

/// A hand correction on one transaction. Beats every rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Override {
    pub txn: TxnId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<CategoryId>,
    #[serde(default)]
    pub tags: Vec<TagId>,
}
