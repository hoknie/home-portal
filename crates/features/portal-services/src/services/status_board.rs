use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, PoisonError, RwLock};

use portal_feature::{StatusChange, StatusObserver};
use portal_model::{ProbeOutcome, ServiceStatus};
use time::OffsetDateTime;

use super::ServiceHistory;
use crate::types::{HistoryRange, HistoryView, ServiceEntry, Tracked};

pub struct StatusBoard {
    started_at: OffsetDateTime,
    tracked: RwLock<HashMap<String, Tracked>>,
    forgotten: Mutex<HashSet<String>>,
    observers: Vec<Arc<dyn StatusObserver>>,
}

impl StatusBoard {
    pub fn watched(
        started_at: OffsetDateTime,
        observers: Vec<Arc<dyn StatusObserver>>,
    ) -> StatusBoard {
        StatusBoard {
            started_at,
            tracked: RwLock::new(HashMap::new()),
            forgotten: Mutex::new(HashSet::new()),
            observers,
        }
    }

    pub fn restore(&self, histories: HashMap<String, ServiceHistory>) {
        let mut tracked = self.tracked.write().unwrap_or_else(PoisonError::into_inner);
        for (id, history) in histories {
            tracked.insert(
                id,
                Tracked {
                    status: ServiceStatus::unknown(self.started_at),
                    history,
                },
            );
        }
    }

    pub fn status(&self, id: &str) -> ServiceStatus {
        self.tracked
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id)
            .map(|tracked| tracked.status.clone())
            .unwrap_or_else(|| ServiceStatus::unknown(self.started_at))
    }

    pub fn history(&self, id: &str, range: HistoryRange, now: OffsetDateTime) -> HistoryView {
        let at = now.unix_timestamp();
        self.tracked
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id)
            .map(|tracked| tracked.history.view(range, at))
            .unwrap_or_else(|| ServiceHistory::default().view(range, at))
    }

    pub fn record(&self, entry: &ServiceEntry, outcome: ProbeOutcome, now: OffsetDateTime) {
        let id = entry.id.as_str();
        let change = {
            let mut tracked = self.tracked.write().unwrap_or_else(PoisonError::into_inner);
            let current = tracked.entry(id.to_string()).or_insert_with(|| Tracked {
                status: ServiceStatus::unknown(self.started_at),
                history: ServiceHistory::default(),
            });
            current.history.record(now.unix_timestamp(), &outcome);
            let next = current.status.after(outcome, now);
            let change =
                (next.state != current.status.state && entry.notifies()).then(|| StatusChange {
                    service: id.to_string(),
                    name: entry.name.clone(),
                    was: current.status.state.name().to_string(),
                    now: next.state.name().to_string(),
                    error: next.last_error.clone(),
                });
            current.status = next;
            change
        };
        if let Some(change) = change {
            for observer in &self.observers {
                observer.changed(&change);
            }
        }
    }

    pub fn forget(&self, id: &str) {
        self.tracked
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(id);
        self.forgotten
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id.to_string());
    }

    pub fn rename(&self, from: &str, to: &str) {
        let mut tracked = self.tracked.write().unwrap_or_else(PoisonError::into_inner);
        if let Some(mut moved) = tracked.remove(from) {
            moved.history.dirty = true;
            tracked.insert(to.to_string(), moved);
        }
        let mut forgotten = self
            .forgotten
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        forgotten.insert(from.to_string());
        forgotten.remove(to);
    }

    pub fn take_dirty(&self) -> Vec<(String, ServiceHistory)> {
        let mut tracked = self.tracked.write().unwrap_or_else(PoisonError::into_inner);
        tracked
            .iter_mut()
            .filter(|(_, tracked)| tracked.history.dirty)
            .map(|(id, tracked)| {
                tracked.history.dirty = false;
                (id.clone(), tracked.history.clone())
            })
            .collect()
    }

    pub fn take_forgotten(&self) -> Vec<String> {
        let tracked = self.tracked.read().unwrap_or_else(PoisonError::into_inner);
        self.forgotten
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .drain()
            .filter(|id| !tracked.contains_key(id))
            .collect()
    }
}
