use super::account::Obligation;
use super::ids::{AccountId, AssetId};
use super::money::Money;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub as_of: NaiveDate,
    pub horizon_start: NaiveDate,
    pub horizon_end: NaiveDate,
    pub opening: Vec<AccountBalance>,
    pub steps: Vec<PlanStep>,
    /// Balances once every step has executed.
    pub projected: Vec<AccountBalance>,
    pub investable: Money,
    #[serde(default)]
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountBalance {
    pub account: AccountId,
    pub balance: Money,
    /// Balance as at this date. Statements close on different days, so a plan is
    /// always built from balances of differing freshness.
    pub as_of: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStep {
    pub seq: u32,
    pub due_by: NaiveDate,
    pub kind: StepKind,
    pub status: StepStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum StepKind {
    Transfer {
        from: AccountId,
        to: AccountId,
        amount: Money,
        reason: String,
    },
    CardPayment {
        from: AccountId,
        card: AccountId,
        amount: Money,
        due_date: NaiveDate,
    },
    Investment {
        from: AccountId,
        asset: AssetId,
        amount: Money,
        reason: String,
    },
    /// Something only the user can do: get an FX quote, raise a transfer limit.
    Manual { text: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StepStatus {
    #[default]
    Pending,
    Done,
    Skipped,
}

/// The planner never returns an infeasible plan silently.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Warning {
    /// Obligations exceed available cash.
    Shortfall {
        amount: Money,
        at_risk: Vec<Obligation>,
    },
    /// A floor had to be violated to meet a due date.
    FloorBreach { account: AccountId, by: Money },
    /// Funding cannot land in time.
    DueDateAtRisk {
        card: AccountId,
        due_date: NaiveDate,
    },
    /// Planning on data older than a full cycle.
    StaleStatement {
        account: AccountId,
        last_seen: NaiveDate,
    },
    /// Approaching the ₹10L LRS limit for the financial year.
    LrsHeadroom { used: Money, remaining: Money },
}
