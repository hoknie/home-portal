use std::collections::HashSet;
use std::sync::{Mutex, PoisonError};

use crate::clients::{GroupRegistry, kill_group};

#[derive(Default)]
pub struct RunningGroups {
    groups: Mutex<HashSet<u32>>,
}

impl RunningGroups {
    pub fn kill_all(&self) {
        let groups: Vec<u32> = self
            .groups
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .copied()
            .collect();
        for group in groups {
            kill_group(group);
        }
    }

    pub fn count(&self) -> usize {
        self.groups
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }
}

impl GroupRegistry for RunningGroups {
    fn started(&self, group: u32) {
        self.groups
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(group);
    }

    fn reaped(&self, group: u32) {
        self.groups
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&group);
    }
}
