use core::{f64::consts::LN_2, num::NonZeroU64};

/// Exponential issuance curve in unit-less steps with a linear portion to
/// complete issuance at step `complete_at`.
#[derive(Debug, Clone)]
pub struct Issuance {
    pub total: u128,
    pub half_life: NonZeroU64,
    pub complete_at: NonZeroU64,
}

impl Issuance {
    pub fn total_issued_by(&self, index: u64) -> u128 {
        self.total - self.unissued_at(index)
    }

    fn unissued_at(&self, index: u64) -> u128 {
        if index >= self.complete_at.get() {
            return 0;
        }
        if index == 0 {
            return self.total;
        }

        (self.total as f64 * self.percent_unissued_at(index)) as u128
    }

    // Exponential curve with a half-life of h = e^(-x * ln2 / h).
    // Linear portion is tangent to the curve such that the line equals zero at
    // `complete_at`.
    fn percent_unissued_at(&self, index: u64) -> f64 {
        let complete_at = self.complete_at.get() as f64;
        let half_life = self.half_life.get() as f64;
        let index = index as f64;

        let linear_after = complete_at - half_life / LN_2;

        let e = |x: f64| (-x * LN_2 / half_life).exp();
        if index > linear_after {
            e(linear_after) - LN_2 / half_life * e(linear_after) * (index - linear_after)
        } else {
            e(index)
        }
    }
}
