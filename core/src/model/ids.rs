use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! hex_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

hex_id!(
    AccountId,
    "Stable identity for one account across every statement it appears in."
);
hex_id!(
    TxnId,
    "Content hash of a transaction; re-importing an overlapping statement yields the same id."
);
hex_id!(
    AssetId,
    "An investable target: a scheme, ticker, or asset bucket."
);
hex_id!(CategoryId, "A spend category.");
hex_id!(TagId, "A free-form tag.");

/// The natural key an account is recognized by across statements.
///
/// Masked account numbers are formatted differently by different statements from the
/// same bank (`XXXXXX1234` vs `****1234`), so only the trailing digits participate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountKey {
    pub institution: String,
    pub kind: super::account::AccountKind,
    pub masked_number: String,
}

impl AccountKey {
    pub fn account_id(&self) -> AccountId {
        let digits: String = self
            .masked_number
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        let mut h = blake3::Hasher::new();
        h.update(self.institution.trim().to_lowercase().as_bytes());
        h.update(b"\x00");
        h.update(format!("{:?}", self.kind).as_bytes());
        h.update(b"\x00");
        h.update(digits.as_bytes());
        AccountId(h.finalize().to_hex()[..16].to_string())
    }
}
