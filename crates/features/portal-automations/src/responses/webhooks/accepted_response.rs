use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AcceptedResponse {
    pub accepted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
}
