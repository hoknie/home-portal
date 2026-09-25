use portal_widget::{WidgetInstance, WidgetSize};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetView {
    pub key: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub id: Option<String>,
    pub title: Option<String>,
    pub settings: Value,
    pub section: Option<String>,
    pub size: WidgetSize,
    pub environments: Option<Vec<String>>,
    pub public: bool,
}

impl WidgetView {
    pub const POSITION_PREFIX: &'static str = "#";

    pub fn key_of(index: usize, instance: &WidgetInstance) -> String {
        instance
            .id
            .clone()
            .unwrap_or_else(|| format!("{}{index}", Self::POSITION_PREFIX))
    }

    pub fn of(key: String, instance: WidgetInstance) -> WidgetView {
        WidgetView {
            key,
            kind: instance.kind,
            id: instance.id,
            title: instance.title,
            settings: instance.settings,
            section: instance.section,
            size: instance.size,
            environments: instance.environments,
            public: instance.public,
        }
    }
}
