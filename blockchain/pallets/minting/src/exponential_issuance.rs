use core::f64::consts::LN_2;
use num_traits::*;

/// Exponential issuance curve in unit-less steps with a linear portion to
/// complete issuance at step `complete_at`.
#[derive(Debug, Clone)]
pub struct Issuance<Balance = u128, Step = u64> {
    pub total: Balance,
    pub half_life: Step,
    pub complete_at: Step,
}

impl<Balance, Step> Issuance<Balance, Step>
where
    Step: num_traits::PrimInt,
    Balance: num_traits::PrimInt,
{
    pub fn total_issued_by(&self, index: Step) -> Balance {
        self.total - self.unissued_at(index)
    }

    fn unissued_at(&self, index: Step) -> Balance {
        if index >= self.complete_at {
            return zero();
        }
        if index == zero() {
            return self.total;
        }

        let total: f64 = cast(self.total).unwrap();
        cast(self.percent_unissued_at(index) * total).unwrap()
    }

    // Exponential curve with a half-life of h = e^(-x * ln2 / h).
    // Linear portion is tangent to the curve such that the line equals zero at
    // `complete_at`.
    fn percent_unissued_at(&self, index: Step) -> f64 {
        let complete_at: f64 = cast(self.complete_at).unwrap();
        let half_life: f64 = cast(self.half_life).unwrap();
        let index: f64 = cast(index).unwrap();

        let linear_after = complete_at - half_life / LN_2;

        let e = |x: f64| (-x * LN_2 / half_life).exp();
        if index > linear_after {
            e(linear_after) - LN_2 / half_life * e(linear_after) * (index - linear_after)
        } else {
            e(index)
        }
    }
}
