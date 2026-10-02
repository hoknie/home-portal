use portal_widget::{ResolvedAppearance, WidgetHeight, WidgetInstance};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct WidgetView {
    pub key: String,
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
    pub environments: Option<Vec<String>>,
    pub public: bool,
}

impl WidgetView {
    pub const POSITION_PREFIX: &'static str = "#";

    pub fn key_of(index: usize) -> String {
        format!("{}{index}", Self::POSITION_PREFIX)
    }

    pub fn of(key: String, instance: WidgetInstance) -> WidgetView {
        let width = instance.columns();
        let position = instance.position();
        let appearance = instance.appearance.resolved();
        WidgetView {
            key,
            kind: instance.kind,
            id: instance.id,
            title: instance.title,
            settings: instance.settings,
            section: instance.section,
            column: position.map(|(column, _)| column),
            row: position.map(|(_, row)| row),
            width,
            height: instance.height,
            appearance,
            environments: instance.environments,
            public: instance.public,
        }
    }
}
