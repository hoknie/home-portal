use serde::Deserialize;
use serde_json::{Map, Value};

use super::RunRequest;
use crate::types::{RawAutomation, RawRun};

#[derive(Debug, Clone, Deserialize)]
pub struct AutomationRequest {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub cooldown_seconds: Option<i64>,
    #[serde(default)]
    pub when: Map<String, Value>,
    #[serde(default)]
    pub run: RunRequest,
}

impl AutomationRequest {
    pub fn into_raw(self) -> RawAutomation {
        RawAutomation {
            id: self.id,
            title: self.title,
            enabled: self.enabled,
            tags: self.tags,
            cooldown_seconds: self.cooldown_seconds,
            when: self
                .when
                .into_iter()
                .filter_map(|(key, value)| Self::toml_of(value).map(|value| (key, value)))
                .collect(),
            run: RawRun {
                script: self.run.script,
                args: self.run.args,
                timeout_seconds: self.run.timeout_seconds,
            },
        }
    }

    fn toml_of(value: Value) -> Option<toml::Value> {
        match value {
            Value::Null => None,
            Value::Bool(flag) => Some(toml::Value::Boolean(flag)),
            Value::Number(number) => number
                .as_i64()
                .map(toml::Value::Integer)
                .or_else(|| number.as_f64().map(toml::Value::Float)),
            Value::String(text) => Some(toml::Value::String(text)),
            Value::Array(items) => Some(toml::Value::Array(
                items.into_iter().filter_map(Self::toml_of).collect(),
            )),
            Value::Object(entries) => Some(toml::Value::Table(
                entries
                    .into_iter()
                    .filter_map(|(key, value)| Self::toml_of(value).map(|value| (key, value)))
                    .collect(),
            )),
        }
    }
}
