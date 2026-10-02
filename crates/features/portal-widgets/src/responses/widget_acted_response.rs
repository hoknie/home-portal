use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct WidgetActedResponse {
    pub run_id: Option<String>,
}
