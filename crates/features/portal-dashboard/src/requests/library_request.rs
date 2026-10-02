use portal_widget::{WidgetAppearance, WidgetHeight, WidgetInstance};
use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryRequest {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default = "LibraryRequest::empty_settings")]
    pub settings: Value,
    #[serde(default)]
    pub environments: Option<Vec<String>>,
    #[serde(default)]
    pub public: bool,
    #[serde(default)]
    pub appearance: WidgetAppearance,
    #[serde(default)]
    pub width: Option<i64>,
    #[serde(default)]
    pub height: WidgetHeight,
}

impl LibraryRequest {
    pub fn empty_settings() -> Value {
        Value::Object(Map::new())
    }

    pub fn into_instance(self) -> WidgetInstance {
        let blank_to_none = |text: Option<String>| {
            text.map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty())
        };
        let appearance = if self.appearance.problems().is_empty() {
            WidgetAppearance::of(self.appearance.resolved())
        } else {
            self.appearance
        };
        WidgetInstance {
            id: blank_to_none(self.id),
            title: blank_to_none(self.title),
            settings: self.settings,
            environments: self.environments,
            public: self.public,
            appearance,
            width: self
                .width
                .filter(|width| *width != i64::from(WidgetInstance::COLUMNS)),
            height: self.height,
            ..WidgetInstance::of(self.kind.trim())
        }
    }
}
