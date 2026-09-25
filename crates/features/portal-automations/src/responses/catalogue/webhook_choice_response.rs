use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WebhookChoiceResponse {
    pub id: String,
    pub name: String,
    pub variables: Vec<String>,
}
