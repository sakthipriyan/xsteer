use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

/// An INR amount. Wraps `Decimal` so cents never round through a float.
///
/// Xsteer is single-currency at the ledger level; foreign holdings are converted at
/// the rate recorded on the transaction, never at display time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Money(pub Decimal);

impl Money {
    pub const ZERO: Money = Money(Decimal::ZERO);

    pub fn new(d: Decimal) -> Self {
        Money(d)
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn is_positive(&self) -> bool {
        self.0 > Decimal::ZERO
    }

    /// The amount above `floor`, or zero when at or below it. Never negative, so
    /// callers can treat it as spendable headroom without a sign check.
    pub fn excess_over(self, floor: Money) -> Money {
        if self.0 > floor.0 {
            Money(self.0 - floor.0)
        } else {
            Money::ZERO
        }
    }

    pub fn min(self, other: Money) -> Money {
        if self.0 <= other.0 {
            self
        } else {
            other
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "₹{}", self.0.round_dp(2))
    }
}

impl Add for Money {
    type Output = Money;
    fn add(self, rhs: Money) -> Money {
        Money(self.0 + rhs.0)
    }
}

impl Sub for Money {
    type Output = Money;
    fn sub(self, rhs: Money) -> Money {
        Money(self.0 - rhs.0)
    }
}

impl Neg for Money {
    type Output = Money;
    fn neg(self) -> Money {
        Money(-self.0)
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, rhs: Money) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, rhs: Money) {
        self.0 -= rhs.0;
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        iter.fold(Money::ZERO, |a, b| a + b)
    }
}
