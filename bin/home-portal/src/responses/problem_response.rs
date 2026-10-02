use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProblemResponse {
    pub file: Option<String>,
    pub field: Option<String>,
    pub message: String,
}
