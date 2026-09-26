use std::sync::{Arc, Mutex, PoisonError, RwLock};

use crate::types::{DnsSettings, DnsState, ZoneBook};

#[derive(Default)]
pub struct Library {
    book: RwLock<Arc<ZoneBook>>,
    settings: RwLock<Arc<DnsSettings>>,
    state: Mutex<DnsState>,
}

impl Library {
    pub fn book(&self) -> Arc<ZoneBook> {
        self.book
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn settings(&self) -> Arc<DnsSettings> {
        self.settings
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn publish(&self, settings: DnsSettings, book: ZoneBook) {
        *self
            .settings
            .write()
            .unwrap_or_else(PoisonError::into_inner) = Arc::new(settings);
        *self.book.write().unwrap_or_else(PoisonError::into_inner) = Arc::new(book);
    }

    pub fn state(&self) -> DnsState {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn update_state(&self, change: impl FnOnce(&mut DnsState)) {
        change(&mut self.state.lock().unwrap_or_else(PoisonError::into_inner));
    }
}
