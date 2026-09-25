use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ScheduleResponse {
    pub timezone: String,
    pub times: Vec<String>,
}
