use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct PreviewStepResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl PreviewStepResponse {
    pub fn of(outcome: &Result<Value, String>) -> PreviewStepResponse {
        match outcome {
            Ok(value) => PreviewStepResponse {
                value: Some(value.clone()),
                error: None,
            },
            Err(message) => PreviewStepResponse {
                value: None,
                error: Some(message.clone()),
            },
        }
    }
}
