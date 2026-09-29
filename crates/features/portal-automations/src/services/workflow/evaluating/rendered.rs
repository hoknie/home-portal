use std::sync::{Arc, Mutex, PoisonError};

use serde_json::Value;

use super::frame::Frame;
use crate::helpers::OPEN;
use crate::types::StepLog;

pub type Collector = Arc<Mutex<StepLog>>;

pub fn collector() -> Collector {
    Arc::new(Mutex::new(StepLog::default()))
}

pub fn collected(collector: &Collector) -> StepLog {
    std::mem::take(&mut *collector.lock().unwrap_or_else(PoisonError::into_inner))
}

pub fn snapshot(collector: &Collector) -> StepLog {
    let log = collector.lock().unwrap_or_else(PoisonError::into_inner);
    StepLog {
        values: log.values.clone(),
        values_dropped: log.values_dropped,
        ..StepLog::default()
    }
}

pub fn record(frame: &Frame, template: &str, value: &Value) {
    if frame.transform_item.is_some() || !template.contains(OPEN) {
        return;
    }
    if let Some(collector) = &frame.rendered {
        collector
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push_value(template, &value.to_string());
    }
}
