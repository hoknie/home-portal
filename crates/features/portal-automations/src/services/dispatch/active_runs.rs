use std::collections::BTreeMap;
use std::sync::{Mutex, PoisonError};
use std::time::Instant;

use time::OffsetDateTime;

use crate::types::{ActiveRun, Pending, RunControl, RunFilter};

#[derive(Default)]
pub struct ActiveRuns {
    runs: Mutex<BTreeMap<u64, ActiveRun>>,
}

impl ActiveRuns {
    pub fn queued(&self, pending: &Pending, at: OffsetDateTime) {
        self.lock()
            .insert(pending.run_id, ActiveRun::queued(pending, at));
    }

    pub fn started(&self, run_id: u64, arguments: Vec<String>, at: OffsetDateTime) {
        if let Some(run) = self.lock().get_mut(&run_id) {
            run.arguments = arguments;
            run.started = Some((at, Instant::now()));
        }
    }

    pub fn remove(&self, run_id: u64) {
        self.lock().remove(&run_id);
    }

    pub fn clear(&self) {
        self.lock().clear();
    }

    pub fn control(&self, run_id: u64) -> Option<RunControl> {
        self.lock().get(&run_id).map(|run| run.control.clone())
    }

    pub fn stopped_by(&self, run_id: u64) -> Option<String> {
        self.lock()
            .get(&run_id)
            .and_then(|run| run.stopped_by.clone())
    }

    pub fn request_stop(&self, run_id: u64, by: &str) -> Option<bool> {
        self.lock().get_mut(&run_id).map(|run| run.request_stop(by))
    }

    pub fn find(&self, run_id: u64) -> Option<ActiveRun> {
        self.lock().get(&run_id).cloned()
    }

    pub fn matching(&self, filter: &RunFilter) -> Vec<ActiveRun> {
        self.lock()
            .values()
            .rev()
            .filter(|run| filter.keeps_active(run))
            .cloned()
            .collect()
    }

    pub fn of_automation(&self, automation: &str) -> Option<ActiveRun> {
        self.lock()
            .values()
            .rev()
            .find(|run| run.automation == automation)
            .cloned()
    }

    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.lock().len()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<u64, ActiveRun>> {
        self.runs.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
