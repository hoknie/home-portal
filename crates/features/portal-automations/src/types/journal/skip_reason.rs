#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    Running,
    Pending,
    Cooldown,
    RateLimit,
    Dropped,
    Removed,
}

impl SkipReason {
    pub fn name(self) -> &'static str {
        match self {
            SkipReason::Running => "running",
            SkipReason::Pending => "pending",
            SkipReason::Cooldown => "cooldown",
            SkipReason::RateLimit => "rate-limit",
            SkipReason::Dropped => "dropped",
            SkipReason::Removed => "removed",
        }
    }
}
