use portal_widget::{ResolvedAppearance, WidgetHeight};
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
    pub column: Option<u8>,
    pub row: Option<u32>,
    pub width: u8,
    pub height: WidgetHeight,
    pub appearance: ResolvedAppearance,
}
