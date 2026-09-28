use std::time::Duration;

use time::OffsetDateTime;

use super::StepOutcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEntry {
    pub path: String,
    pub step: String,
    pub label: String,
    pub kind: String,
    pub iteration: Option<usize>,
    pub outcome: StepOutcome,
    pub started_at: OffsetDateTime,
    pub duration: Duration,
    pub detail: String,
    pub output: Option<String>,
    pub shape: Option<String>,
}

impl TraceEntry {
    pub const LONGEST_DETAIL: usize = 500;
    pub const LONGEST_OUTPUT: usize = 4 * 1024;
}
