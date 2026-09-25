use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FieldResponse {
    pub name: String,
    pub sample: String,
}
