use serde::{Deserialize, Serialize};

use super::StoredTraceEntry;
use crate::types::Trace;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredTrace {
    pub entries: Vec<StoredTraceEntry>,
    #[serde(default)]
    pub dropped: usize,
}

impl StoredTrace {
    pub fn of(trace: &Trace) -> StoredTrace {
        StoredTrace {
            entries: trace.entries.iter().map(StoredTraceEntry::of).collect(),
            dropped: trace.dropped,
        }
    }

    pub fn into_trace(self) -> Trace {
        Trace {
            entries: self
                .entries
                .into_iter()
                .filter_map(StoredTraceEntry::into_entry)
                .collect(),
            dropped: self.dropped,
            log_bytes: 0,
        }
    }
}
