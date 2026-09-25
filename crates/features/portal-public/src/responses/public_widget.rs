use portal_widget::WidgetSize;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicWidget {
    #[serde(rename = "type")]
    pub kind: String,
    pub id: Option<String>,
    pub title: Option<String>,
    pub settings: Value,
    pub section: Option<String>,
    pub size: WidgetSize,
}
