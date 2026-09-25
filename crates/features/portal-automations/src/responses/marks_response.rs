use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MarksResponse {
    pub enabled: bool,
    pub tags: Vec<String>,
}
