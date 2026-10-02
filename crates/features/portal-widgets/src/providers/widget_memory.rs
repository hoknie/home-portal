use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, PoisonError};

use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq)]
pub struct Remembered {
    pub data: Value,
    pub at: OffsetDateTime,
    pub source: String,
}

#[derive(Default)]
pub struct WidgetMemory {
    data: Mutex<HashMap<String, Remembered>>,
    rendered_with: Mutex<HashMap<String, String>>,
    rerun: Mutex<HashSet<String>>,
}

impl WidgetMemory {
    pub fn remembered(&self, id: &str) -> Option<Remembered> {
        self.data
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id)
            .cloned()
    }

    pub fn remember(&self, id: &str, remembered: Remembered) {
        self.data
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id.to_string(), remembered);
    }

    pub fn rendered(&self, id: &str, settings: String) {
        self.rendered_with
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id.to_string(), settings);
    }

    pub fn rendered_with(&self, id: &str) -> Option<String> {
        self.rendered_with
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id)
            .cloned()
    }

    pub fn ask_rerun(&self, id: &str) {
        self.rerun
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id.to_string());
    }

    pub fn rerun_asked(&self, id: &str) -> bool {
        self.rerun
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .contains(id)
    }

    pub fn take_rerun(&self, id: &str) -> bool {
        self.rerun
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(id)
    }
}
