use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RawCondition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all: Option<Vec<RawCondition>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub any: Option<Vec<RawCondition>>,
}
