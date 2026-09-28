use serde::{Deserialize, Serialize};

use super::RawStep;
use crate::types::RawInput;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RawWorkflow {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<i64>,
    #[serde(default)]
    pub inputs: Vec<RawInput>,
    #[serde(default)]
    pub steps: Vec<RawStep>,
}
