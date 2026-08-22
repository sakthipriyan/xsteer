//! Statement files → normalized entities.
//!
//! Parsing itself is `xfina`'s job. What lives here is everything `xfina` cannot do,
//! because it sees one file at a time and Xsteer sees a history:
//!
//! - resolving [`AccountKey`](crate::model::AccountKey) to a stable account identity
//! - deduplicating transactions across statements with overlapping periods
//! - lifting credit card statement totals and due dates into [`CardState`]
//! - detecting recurring credits that look like salary
//!
//! Phase 1. Not yet implemented.
