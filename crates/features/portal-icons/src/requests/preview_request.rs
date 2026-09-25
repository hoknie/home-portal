use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PreviewRequest {
    pub icon: String,
    #[serde(default)]
    pub url: Option<String>,
}
