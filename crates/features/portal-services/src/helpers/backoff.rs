use std::time::Duration;

pub const LONGEST_WAIT: Duration = Duration::from_secs(300);

pub fn next_wait(every: Duration, consecutive_failures: u32) -> Duration {
    let ceiling = every.max(LONGEST_WAIT);
    let factor = 2u32.saturating_pow(consecutive_failures.min(16));
    every.saturating_mul(factor).min(ceiling)
}
