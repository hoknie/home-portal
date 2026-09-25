use super::{HistoryRange, LatencyPoint, Transition, Uptime};

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryView {
    pub range: HistoryRange,
    pub from: i64,
    pub to: i64,
    pub uptime: Vec<Uptime>,
    pub points: Vec<LatencyPoint>,
    pub transitions: Vec<Transition>,
}
