//! The cashflow solver: balances + obligations + policies → an ordered to-do list.
//!
//! The algorithm is specified step by step in `docs/DESIGN.md` §4. It is deterministic
//! — ties break on `AccountId` — so the same vault always yields the same plan, which
//! is what makes snapshot testing possible.
//!
//! Phase 4. Not yet implemented.
