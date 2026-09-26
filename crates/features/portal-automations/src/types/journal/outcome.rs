#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Succeeded,
    Failed,
    TimedOut,
    Stopped,
    Skipped,
    Refused,
}

impl Outcome {
    pub const ALL: [Outcome; 6] = [
        Outcome::Succeeded,
        Outcome::Failed,
        Outcome::TimedOut,
        Outcome::Stopped,
        Outcome::Skipped,
        Outcome::Refused,
    ];

    pub fn of(name: &str) -> Option<Outcome> {
        Self::ALL.into_iter().find(|outcome| outcome.name() == name)
    }

    pub fn name(self) -> &'static str {
        match self {
            Outcome::Succeeded => "succeeded",
            Outcome::Failed => "failed",
            Outcome::TimedOut => "timed-out",
            Outcome::Stopped => "stopped",
            Outcome::Skipped => "skipped",
            Outcome::Refused => "refused",
        }
    }
}
