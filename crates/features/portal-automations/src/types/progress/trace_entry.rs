use std::time::Duration;

use time::OffsetDateTime;

use super::{StepOutcome, Streams};
use crate::types::{LogLevel, StepLog};

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
    pub streams: Option<Streams>,
    pub log: StepLog,
    pub item: Option<String>,
    pub level: Option<LogLevel>,
}

impl TraceEntry {
    pub const LONGEST_DETAIL: usize = 500;
    pub const LONGEST_OUTPUT: usize = 4 * 1024;
    pub const LONGEST_ITEM: usize = 100;
}
