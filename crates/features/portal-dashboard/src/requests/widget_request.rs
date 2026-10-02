use portal_widget::WidgetHeight;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct WidgetRequest {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub widget: Option<String>,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub column: Option<i64>,
    #[serde(default)]
    pub row: Option<i64>,
    #[serde(default = "WidgetRequest::full_width")]
    pub width: i64,
    #[serde(default)]
    pub height: WidgetHeight,
}

impl WidgetRequest {
    pub fn full_width() -> i64 {
        12
    }
}
