use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct WebhookChoiceResponse {
    pub id: String,
    pub name: String,
    pub variables: Vec<String>,
    pub action: String,
}
