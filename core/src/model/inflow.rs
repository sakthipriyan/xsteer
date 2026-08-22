use super::ids::AccountId;
use super::money::Money;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Money arriving from outside. Salary is the canonical case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inflow {
    pub name: String,
    pub into: AccountId,
    pub amount: Money,
    pub on: NaiveDate,
    pub recurrence: Recurrence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Recurrence {
    Once,
    MonthlyOn { day: u8 },
}
