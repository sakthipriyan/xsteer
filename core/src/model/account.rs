use super::ids::AccountId;
use super::money::Money;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AccountKind {
    Savings,
    Current,
    CreditCard,
    Brokerage,
    MutualFund,
}

impl AccountKind {
    /// Only deposit accounts hold cash a plan can move.
    pub fn is_deposit(self) -> bool {
        matches!(self, AccountKind::Savings | AccountKind::Current)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: AccountId,
    /// Full institution name, per the xfina convention: "HDFC Bank", not "HDFC".
    pub institution: String,
    pub kind: AccountKind,
    pub masked_number: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<Policy>,
}

/// What an account is *for*. Drives every funding decision the planner makes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    pub role: Role,
    /// Never draw the account below this: minimum balance plus cash buffer.
    pub floor: Money,
    /// Desired steady-state balance. Topped up after obligations are met.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Money>,
    pub sweep: Sweep,
    #[serde(default)]
    pub obligations: Vec<Obligation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Salary,
    Spend,
    Medical,
    Travel,
    Investment,
    Buffer,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Sweep {
    /// Leave any excess where it is.
    Nothing,
    /// Move excess above `target` into another account.
    To { account: AccountId },
    /// Release excess above `target` to the allocator as investable surplus.
    ToInvestable,
}

/// A claim against an account, in the order the planner must satisfy them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Obligation {
    /// Pay this card's statement due. Deliberately a *reference*, not an amount —
    /// re-importing the card statement updates the plan with no edit here.
    CreditCardDue { card: AccountId },
    /// Recurring outflow on a day of the month: rent, EMI, a bank mandate.
    FixedExpense {
        name: String,
        amount: Money,
        day: u8,
    },
    /// A one-off with a date.
    PlannedExpense {
        name: String,
        amount: Money,
        due: chrono::NaiveDate,
    },
    /// Earmarked cash the planner may never spend.
    Reserve { name: String, amount: Money },
}
