use portal_feature::PortalEvent;

use crate::types::Automation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    pub run_id: u64,
    pub automation: Automation,
    pub event: PortalEvent,
    pub by: Option<String>,
}

impl Pending {
    pub fn manual(&self) -> bool {
        self.by.is_some()
    }
}
