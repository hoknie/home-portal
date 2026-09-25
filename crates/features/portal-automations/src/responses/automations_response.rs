use serde::Serialize;

use super::AutomationResponse;

#[derive(Debug, Clone, Serialize)]
pub struct AutomationsResponse {
    pub automations: Vec<AutomationResponse>,
}
