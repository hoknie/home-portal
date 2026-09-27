use std::sync::Arc;

use portal_config::{ConfigStore, Revisioned};

use crate::services::Views;
use crate::types::AutomationView;

#[derive(Clone)]
pub struct ListAutomations {
    configuration: Arc<ConfigStore>,
    views: Views,
}

impl ListAutomations {
    pub fn new(configuration: Arc<ConfigStore>, views: Views) -> ListAutomations {
        ListAutomations {
            configuration,
            views,
        }
    }

    pub fn run(&self) -> Revisioned<Vec<AutomationView>> {
        let snapshot = self.configuration.read();
        Revisioned::new(
            self.views.automations(&snapshot.document),
            snapshot.revision,
        )
    }
}
