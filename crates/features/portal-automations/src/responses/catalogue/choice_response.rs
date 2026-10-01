use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ChoiceResponse {
    pub id: String,
    pub name: String,
}
