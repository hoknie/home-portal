use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use time::OffsetDateTime;

use crate::types::{Finding, PermissionCode};

#[derive(Clone, Default)]
pub struct PermissionBoard {
    records: Arc<Mutex<BTreeMap<PermissionCode, (Finding, OffsetDateTime)>>>,
    asking: Arc<Mutex<BTreeSet<PermissionCode>>>,
}

impl PermissionBoard {
    pub fn record(&self, code: PermissionCode, finding: Finding) {
        held(&self.records).insert(code, (finding, OffsetDateTime::now_utc()));
    }

    pub fn get(&self, code: &PermissionCode) -> Option<(Finding, OffsetDateTime)> {
        held(&self.records).get(code).copied()
    }

    pub fn begin(&self, code: &PermissionCode) -> bool {
        held(&self.asking).insert(code.clone())
    }

    pub fn end(&self, code: &PermissionCode) {
        held(&self.asking).remove(code);
    }

    pub fn asking(&self, code: &PermissionCode) -> bool {
        held(&self.asking).contains(code)
    }
}

fn held<T>(lock: &Mutex<T>) -> MutexGuard<'_, T> {
    lock.lock().unwrap_or_else(PoisonError::into_inner)
}
