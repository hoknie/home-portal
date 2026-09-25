use std::collections::VecDeque;
use std::sync::{Mutex, PoisonError};

use crate::types::{Catalogue, RunFilter, RunRecord};

#[derive(Default)]
pub struct Journal {
    records: Mutex<VecDeque<RunRecord>>,
    unwritten: Mutex<Vec<RunRecord>>,
}

impl Journal {
    pub const KEPT: usize = 200;

    pub fn record(&self, record: RunRecord) {
        let mut records = self.records.lock().unwrap_or_else(PoisonError::into_inner);
        let manual = |candidate: &RunRecord| {
            candidate
                .fields
                .iter()
                .any(|(key, value)| key == Catalogue::RUN_MANUAL_FIELD && value == "true")
        };
        let incoming_manual = manual(&record);
        let latest = records
            .iter_mut()
            .find(|existing| existing.automation == record.automation);
        if let Some(latest) = latest
            .filter(|latest| !incoming_manual && !manual(latest) && latest.merges_with(&record))
        {
            latest.seen.count += 1;
            latest.seen.last_at = record.seen.started_at;
            let merged = latest.clone();
            self.unwritten
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(merged);
            return;
        }
        tracing::info!(
            automation = %record.automation,
            outcome = record.result.outcome.name(),
            reason = record.result.reason.as_deref().unwrap_or_default(),
            exit_code = ?record.result.exit_code,
            duration_milliseconds = record.result.duration.as_millis() as u64,
            "an automation ran"
        );
        self.unwritten
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(record.clone());
        records.push_front(record);
        records.truncate(Self::KEPT);
    }

    pub fn restore(&self, newest_first: Vec<RunRecord>) {
        let mut records = self.records.lock().unwrap_or_else(PoisonError::into_inner);
        records.clear();
        records.extend(newest_first.into_iter().take(Self::KEPT));
    }

    pub fn take_unwritten(&self) -> Vec<RunRecord> {
        std::mem::take(
            &mut *self
                .unwritten
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        )
    }

    pub fn snapshot(&self) -> Vec<RunRecord> {
        self.matching(&RunFilter::default())
    }

    pub fn highest_id(&self) -> u64 {
        self.records
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .map(|record| record.id)
            .max()
            .unwrap_or(0)
    }

    #[cfg(test)]
    pub fn runs(&self, automation: Option<&str>) -> Vec<RunRecord> {
        self.matching(&RunFilter {
            automation: automation.map(str::to_string),
            ..RunFilter::default()
        })
    }

    pub fn matching(&self, filter: &RunFilter) -> Vec<RunRecord> {
        self.records
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .filter(|record| filter.keeps(record))
            .cloned()
            .collect()
    }

    pub fn last_of(&self, automation: &str) -> Option<RunRecord> {
        self.records
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .find(|record| record.automation == automation)
            .cloned()
    }
}
