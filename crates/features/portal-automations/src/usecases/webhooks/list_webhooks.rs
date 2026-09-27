use std::sync::Arc;

use portal_config::{ConfigStore, Revisioned};

use crate::services::Views;
use crate::types::WebhookView;

#[derive(Clone)]
pub struct ListWebhooks {
    configuration: Arc<ConfigStore>,
    views: Views,
}

impl ListWebhooks {
    pub fn new(configuration: Arc<ConfigStore>, views: Views) -> ListWebhooks {
        ListWebhooks {
            configuration,
            views,
        }
    }

    pub fn run(&self) -> Revisioned<Vec<WebhookView>> {
        let snapshot = self.configuration.read();
        Revisioned::new(self.views.webhooks(&snapshot.document), snapshot.revision)
    }
}
