use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ScheduleResponse {
    pub timezone: String,
    pub times: Vec<String>,
}
