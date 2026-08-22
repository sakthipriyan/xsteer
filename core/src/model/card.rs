use super::ids::AccountId;
use super::money::Money;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardState {
    pub id: AccountId,
    /// Latest imported statement. Absent until the first statement is uploaded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statement: Option<Statement>,
    /// Spends after the statement closed. Not due this cycle, but the planner
    /// reports it so a large unbilled balance is never a surprise next month.
    #[serde(default)]
    pub unbilled: Money,
    pub paid_from: AccountId,
    pub cycle: Cycle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statement {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub total_due: Money,
    pub min_due: Money,
    pub due_date: NaiveDate,
}

/// Enough to project the *next* due date before that statement arrives, so a plan
/// built mid-cycle still reserves cash for a card whose statement has not landed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cycle {
    /// Day of month the statement closes.
    pub statement_day: u8,
    /// Days between statement close and payment due.
    pub grace_days: u16,
}
