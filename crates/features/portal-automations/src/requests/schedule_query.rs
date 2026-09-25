use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleQuery {
    #[serde(default)]
    pub cron: String,
}
