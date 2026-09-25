use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SectionRequest {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
}
