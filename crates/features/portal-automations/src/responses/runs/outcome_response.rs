use serde::Serialize;

use super::OutputResponse;

#[derive(Debug, Clone, Serialize)]
pub struct OutcomeResponse {
    pub result: String,
    pub exit_code: Option<i32>,
    pub reason: Option<String>,
    pub duration_milliseconds: u64,
    pub count: u32,
    pub last_at: String,
    pub stdout: OutputResponse,
    pub stderr: OutputResponse,
}
