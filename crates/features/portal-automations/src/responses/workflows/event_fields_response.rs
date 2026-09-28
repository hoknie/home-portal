use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct EventFieldsResponse {
    pub name: String,
    pub fields: Vec<String>,
}
