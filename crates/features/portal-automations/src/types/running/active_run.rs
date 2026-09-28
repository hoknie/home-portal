use std::sync::{Arc, Mutex};
use std::time::Instant;

use time::OffsetDateTime;
use tokio::sync::watch;

use super::{Invocation, Pending, RunControl};
use crate::types::Trace;

#[derive(Debug, Clone)]
pub struct ActiveRun {
    pub run_id: u64,
    pub automation: String,
    pub fields: Vec<(String, String)>,
    pub arguments: Vec<String>,
    pub admitted_at: OffsetDateTime,
    pub started: Option<(OffsetDateTime, Instant)>,
    pub stopped_by: Option<String>,
    pub control: RunControl,
    pub workflow: Option<String>,
    pub trace: Arc<Mutex<Trace>>,
    stop: Arc<watch::Sender<bool>>,
}

impl ActiveRun {
    pub fn queued(pending: &Pending, at: OffsetDateTime) -> ActiveRun {
        let (stop, control) = RunControl::new();
        ActiveRun {
            run_id: pending.run_id,
            automation: pending.automation.id.clone(),
            fields: Invocation::fields_of(pending),
            arguments: Vec::new(),
            admitted_at: at,
            started: None,
            stopped_by: None,
            control,
            workflow: pending
                .automation
                .workflow
                .as_ref()
                .map(|call| call.id.clone()),
            trace: Arc::new(Mutex::new(Trace::default())),
            stop: Arc::new(stop),
        }
    }

    pub fn running(&self) -> bool {
        self.started.is_some()
    }

    pub fn request_stop(&mut self, by: &str) -> bool {
        if self.stopped_by.is_some() {
            return false;
        }
        self.stopped_by = Some(by.to_string());
        self.stop.send_replace(true);
        true
    }
}
