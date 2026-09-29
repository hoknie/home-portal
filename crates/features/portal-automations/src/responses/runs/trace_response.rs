use serde::Serialize;

use super::TraceEntryResponse;
use time::OffsetDateTime;

use crate::types::Trace;

#[derive(Debug, Clone, Serialize)]
pub struct TraceResponse {
    pub entries: Vec<TraceEntryResponse>,
    pub dropped: usize,
}

impl TraceResponse {
    pub fn of(trace: &Trace, now: Option<OffsetDateTime>) -> TraceResponse {
        TraceResponse {
            entries: trace
                .entries
                .iter()
                .map(|entry| TraceEntryResponse::of(entry, now))
                .collect(),
            dropped: trace.dropped,
        }
    }
}
