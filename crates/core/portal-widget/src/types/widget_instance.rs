use portal_model::Environment;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::WidgetSize;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetInstance {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default = "WidgetInstance::empty_settings")]
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

impl WidgetInstance {
    pub fn empty_settings() -> Value {
        Value::Object(Map::new())
    }

    pub fn of(kind: &str) -> WidgetInstance {
        WidgetInstance {
            kind: kind.to_string(),
            id: None,
            title: None,
            settings: Self::empty_settings(),
            environments: None,
            public: false,
            section: None,
            size: WidgetSize::Full,
        }
    }

    pub fn visible_to(&self, environment: &Environment) -> bool {
        match &self.environments {
            None => true,
            Some(names) => names.iter().any(|name| name == environment.as_str()),
        }
    }

    pub fn public_in(&self, environment: &Environment) -> bool {
        self.public && self.visible_to(environment)
    }
}
