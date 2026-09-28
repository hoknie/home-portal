use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RawInput {
    Name(String),
    Declared {
        #[serde(default)]
        name: String,
        #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
        input_type: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        default: Option<Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

impl RawInput {
    pub fn name(&self) -> &str {
        match self {
            RawInput::Name(name) | RawInput::Declared { name, .. } => name,
        }
    }
}
