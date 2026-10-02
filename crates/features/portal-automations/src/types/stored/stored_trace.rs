use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::StoredTraceEntry;
use crate::types::Trace;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredTrace {
    pub entries: Vec<StoredTraceEntry>,
    #[serde(default)]
    pub dropped: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outputs: Option<BTreeMap<String, Value>>,
}

impl StoredTrace {
    pub fn of(trace: &Trace) -> StoredTrace {
        StoredTrace {
            entries: trace.entries.iter().map(StoredTraceEntry::of).collect(),
            dropped: trace.dropped,
            outputs: trace.outputs.clone(),
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
            output_bytes: 0,
            outputs: self.outputs,
        }
    }
}
