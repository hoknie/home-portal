use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawMarks {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub tags: Vec<String>,
}
