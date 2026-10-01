use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MarksResponse {
    pub enabled: bool,
    pub tags: Vec<String>,
}
