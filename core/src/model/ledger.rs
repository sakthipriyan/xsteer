use super::ids::{AccountId, CategoryId, TagId, TxnId};
use super::money::Money;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Direction {
    Debit,
    Credit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Txn {
    pub id: TxnId,
    pub account: AccountId,
    pub date: NaiveDate,
    pub amount: Money,
    pub direction: Direction,
    pub narration: String,
    /// Running balance after this transaction, where the statement supplies one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<CategoryId>,
    #[serde(default)]
    pub tags: Vec<TagId>,
}

impl Txn {
    /// Signed effect on the account balance.
    pub fn signed(&self) -> Money {
        match self.direction {
            Direction::Credit => self.amount,
            Direction::Debit => -self.amount,
        }
    }
}

/// Identity of a transaction, computed from its content so that re-importing an
/// overlapping statement is idempotent.
///
/// The running balance participates because banks legitimately emit two identical
/// same-day same-amount rows; the balance is what tells them apart. Statements with no
/// running balance fall back to the row's ordinal within its day.
pub fn txn_id(
    account: &AccountId,
    date: NaiveDate,
    amount: Money,
    direction: Direction,
    narration: &str,
    balance: Option<Money>,
    ordinal_in_day: u32,
) -> TxnId {
    let mut h = blake3::Hasher::new();
    h.update(account.0.as_bytes());
    h.update(b"\x00");
    h.update(date.to_string().as_bytes());
    h.update(b"\x00");
    h.update(amount.0.to_string().as_bytes());
    h.update(b"\x00");
    h.update(format!("{direction:?}").as_bytes());
    h.update(b"\x00");
    h.update(normalize_narration(narration).as_bytes());
    h.update(b"\x00");
    match balance {
        Some(b) => h.update(b.0.to_string().as_bytes()),
        None => h.update(format!("#{ordinal_in_day}").as_bytes()),
    };
    TxnId(h.finalize().to_hex()[..16].to_string())
}

/// Collapse whitespace and case so cosmetic reformatting between statement versions
/// does not produce a duplicate transaction.
pub fn normalize_narration(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
