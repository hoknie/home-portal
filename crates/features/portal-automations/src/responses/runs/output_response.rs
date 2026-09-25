use serde::Serialize;

use crate::types::Tail;

#[derive(Debug, Clone, Serialize)]
pub struct OutputResponse {
    pub tail: String,
    pub bytes: u64,
    pub truncated: bool,
}

impl OutputResponse {
    pub fn of(tail: &Tail) -> OutputResponse {
        OutputResponse {
            tail: tail.text(),
            bytes: tail.total,
            truncated: tail.truncated(),
        }
    }
}
