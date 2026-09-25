use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ChoiceResponse {
    pub id: String,
    pub name: String,
}
