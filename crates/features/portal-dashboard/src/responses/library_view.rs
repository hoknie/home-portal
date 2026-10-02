use portal_widget::{ResolvedAppearance, WidgetHeight, WidgetInstance};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LibraryWidgetView {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub title: Option<String>,
    #[schemars(with = "serde_json::Map<String, Value>")]
    pub settings: Value,
    pub environments: Option<Vec<String>>,
    pub public: bool,
    pub appearance: ResolvedAppearance,
    pub width: u8,
    pub height: WidgetHeight,
    pub placed: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LibraryResponse {
    pub widgets: Vec<LibraryWidgetView>,
}

impl LibraryWidgetView {
    pub fn of(instance: WidgetInstance, placed: usize) -> LibraryWidgetView {
        let width = instance.columns();
        LibraryWidgetView {
            id: instance.id.unwrap_or_default(),
            appearance: instance.appearance.resolved(),
            width,
            height: instance.height,
            kind: instance.kind,
            title: instance.title,
            settings: instance.settings,
            environments: instance.environments,
            public: instance.public,
            placed,
        }
    }
}
