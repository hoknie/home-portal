use portal_widget::WidgetSize;
use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Deserialize)]
pub struct WidgetRequest {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default = "WidgetRequest::empty_settings")]
    pub settings: Value,
    #[serde(default)]
    pub environments: Option<Vec<String>>,
    #[serde(default)]
    pub public: bool,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub size: WidgetSize,
}

impl WidgetRequest {
    pub fn empty_settings() -> Value {
        Value::Object(Map::new())
    }
}
