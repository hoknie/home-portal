use super::HistoryRange;

#[derive(Debug, Clone, PartialEq)]
pub struct Uptime {
    pub range: HistoryRange,
    pub ratio: Option<f64>,
    pub covered_seconds: u64,
}
