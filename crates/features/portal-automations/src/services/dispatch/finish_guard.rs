use std::sync::Arc;

use super::Gatekeeper;

pub struct FinishGuard {
    pub gatekeeper: Arc<Gatekeeper>,
    pub automation: String,
}

impl Drop for FinishGuard {
    fn drop(&mut self) {
        self.gatekeeper.finished(&self.automation);
    }
}
