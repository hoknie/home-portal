use serde::Serialize;

use super::FieldResponse;

#[derive(Debug, Clone, Serialize)]
pub struct EventResponse {
    pub name: String,
    pub fields: Vec<FieldResponse>,
    pub filters: Vec<String>,
}
