use schemars::JsonSchema;
use serde::Serialize;

use super::AutomationResponse;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AutomationsResponse {
    pub automations: Vec<AutomationResponse>,
}
