use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use super::Block;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCustom {
    #[serde(default)]
    pub source: Option<RawSource>,
    #[serde(default)]
    pub refresh_seconds: Option<i64>,
    #[serde(default)]
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSource {
    #[serde(default)]
    pub workflow: Option<String>,
    #[serde(default)]
    pub inputs: BTreeMap<String, Value>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub timeout_seconds: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAction {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub automation: Option<String>,
    #[serde(default)]
    pub fields: BTreeMap<String, String>,
    #[serde(default)]
    pub workflow: Option<String>,
    #[serde(default)]
    pub inputs: BTreeMap<String, Value>,
    #[serde(default)]
    pub refresh: Option<bool>,
    #[serde(default)]
    pub link: Option<String>,
}
