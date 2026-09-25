use crate::types::Schedule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronFilter {
    pub expression: String,
    pub schedule: Schedule,
}
