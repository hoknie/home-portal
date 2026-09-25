use serde::Deserialize;

use super::RawRun;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawAutomation {
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
    pub when: toml::Table,
    #[serde(default)]
    pub run: RawRun,
}
