use serde::Serialize;

use super::TraceEntryResponse;
use crate::types::Trace;

#[derive(Debug, Clone, Serialize)]
pub struct TraceResponse {
    pub entries: Vec<TraceEntryResponse>,
    pub dropped: usize,
}

impl TraceResponse {
    pub fn of(trace: &Trace) -> TraceResponse {
        TraceResponse {
            entries: trace.entries.iter().map(TraceEntryResponse::of).collect(),
            dropped: trace.dropped,
        }
    }
}
