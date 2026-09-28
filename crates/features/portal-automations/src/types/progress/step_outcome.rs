#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    Running,
    Succeeded,
    Failed,
    Skipped,
    Stopped,
    TimedOut,
}

impl StepOutcome {
    pub const ALL: [StepOutcome; 6] = [
        StepOutcome::Running,
        StepOutcome::Succeeded,
        StepOutcome::Failed,
        StepOutcome::Skipped,
        StepOutcome::Stopped,
        StepOutcome::TimedOut,
    ];

    pub fn name(self) -> &'static str {
        match self {
            StepOutcome::Running => "running",
            StepOutcome::Succeeded => "succeeded",
            StepOutcome::Failed => "failed",
            StepOutcome::Skipped => "skipped",
            StepOutcome::Stopped => "stopped",
            StepOutcome::TimedOut => "timed-out",
        }
    }

    pub fn of(name: &str) -> Option<StepOutcome> {
        Self::ALL.into_iter().find(|outcome| outcome.name() == name)
    }
}
