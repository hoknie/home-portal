use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::RawCondition;
use crate::types::{RawNumber, RawOperation};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RawStep {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<RawCondition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub then: Option<Vec<RawStep>>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "else")]
    pub otherwise: Option<Vec<RawStep>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<RawNumber>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub for_each: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "while")]
    pub while_condition: Option<RawCondition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<RawNumber>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<toml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branches: Option<Vec<Vec<RawStep>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputs: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variable: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<RawNumber>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<RawNumber>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_on_error: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_sample: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operations: Option<Vec<RawOperation>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
}
