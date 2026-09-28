use std::sync::Arc;

use portal_config::{ConfigStore, Revisioned};

use crate::services::Views;
use crate::types::WorkflowView;

#[derive(Clone)]
pub struct ListWorkflows {
    configuration: Arc<ConfigStore>,
    views: Views,
}

impl ListWorkflows {
    pub fn new(configuration: Arc<ConfigStore>, views: Views) -> ListWorkflows {
        ListWorkflows {
            configuration,
            views,
        }
    }

    pub fn run(&self) -> Revisioned<Vec<WorkflowView>> {
        let snapshot = self.configuration.read();
        Revisioned::new(self.views.workflows(&snapshot.document), snapshot.revision)
    }
}
