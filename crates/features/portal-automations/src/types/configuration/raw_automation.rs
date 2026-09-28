use std::collections::BTreeMap;

use serde::Deserialize;

use super::RawRun;
use crate::types::InputValue;

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
    pub run: Option<RawRun>,
    #[serde(default)]
    pub workflow: Option<String>,
    #[serde(default)]
    pub inputs: Option<BTreeMap<String, InputValue>>,
}
