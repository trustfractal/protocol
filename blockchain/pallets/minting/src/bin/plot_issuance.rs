use core::convert::TryInto;
use fractal_minting::Issuance;

/// Utility to generate issuance curves with hard-coded parameters for plotting
/// with external tools.
fn main() {
    let issuance = Issuance {
        total: 400_000_000,
        half_life: 52560000,
        complete_at: 52560000 * 3,
    };
    let step_by = 14400;

    let mut current = 0;
    while current <= issuance.complete_at {
        println!(
            "{},{}",
            current / step_by,
            issuance.total_issued_by(current)
        );
        current += step_by;
    }
}
