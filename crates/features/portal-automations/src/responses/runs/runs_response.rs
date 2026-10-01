use schemars::JsonSchema;
use serde::Serialize;

use super::RunResponse;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RunsResponse {
    pub runs: Vec<RunResponse>,
}
