use portal_widget::SectionAppearance;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SectionRequest {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub appearance: SectionAppearance,
}
