use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct FieldResponse {
    pub name: String,
    pub sample: String,
}
