use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::RawCondition;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RawOperation {
    #[serde(default)]
    pub op: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "where")]
    pub condition: Option<RawCondition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operations: Option<Vec<RawOperation>>,
}
