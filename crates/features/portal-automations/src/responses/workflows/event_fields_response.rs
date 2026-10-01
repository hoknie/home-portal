use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct EventFieldsResponse {
    pub name: String,
    pub fields: Vec<String>,
}
