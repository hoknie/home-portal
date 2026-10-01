use portal_widget::WidgetSize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PublicWidget {
    #[serde(rename = "type")]
    pub kind: String,
    pub id: Option<String>,
    pub title: Option<String>,
    #[schemars(with = "serde_json::Map<String, Value>")]
    pub settings: Value,
    pub section: Option<String>,
    pub size: WidgetSize,
}
