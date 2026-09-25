use std::time::Duration;

use super::Tail;
use crate::types::Outcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finished {
    pub outcome: Outcome,
    pub exit_code: Option<i32>,
    pub reason: Option<String>,
    pub duration: Duration,
    pub stdout: Tail,
    pub stderr: Tail,
}
