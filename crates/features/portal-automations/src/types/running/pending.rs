use portal_feature::PortalEvent;

use crate::types::Automation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    pub run_id: u64,
    pub automation: Automation,
    pub event: PortalEvent,
    pub by: Option<String>,
    pub origin: Vec<String>,
}

impl Pending {
    pub fn manual(&self) -> bool {
        self.by.is_some()
    }

    pub fn child(&self) -> bool {
        !self.origin.is_empty()
    }

    pub fn chain(&self) -> Vec<String> {
        let mut chain = self.origin.clone();
        chain.push(self.automation.id.clone());
        chain
    }
}
