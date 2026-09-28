use crate::types::StepOutcome;

use super::StepLog;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryEnd {
    pub outcome: StepOutcome,
    pub detail: String,
    pub output: Option<String>,
    pub shape: Option<String>,
    pub log: StepLog,
}
