use serde::{Deserialize, Serialize};

use super::StoredTail;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredResult {
    pub outcome: String,
    pub exit_code: Option<i32>,
    pub reason: Option<String>,
    pub duration_milliseconds: u64,
    pub stdout: StoredTail,
    pub stderr: StoredTail,
}
