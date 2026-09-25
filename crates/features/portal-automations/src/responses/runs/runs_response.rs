use serde::Serialize;

use super::RunResponse;

#[derive(Debug, Clone, Serialize)]
pub struct RunsResponse {
    pub runs: Vec<RunResponse>,
}
