use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::ApiError;

use crate::repositories::{origin, position, remove};
use crate::services::Views;
use crate::types::{Automation, AutomationView};

#[derive(Clone)]
pub struct DeleteAutomation {
    configuration: Arc<ConfigStore>,
    views: Views,
}

impl DeleteAutomation {
    pub fn new(configuration: Arc<ConfigStore>, views: Views) -> DeleteAutomation {
        DeleteAutomation {
            configuration,
            views,
        }
    }

    pub async fn run(
        &self,
        id: &str,
        revision: &Revision,
    ) -> Result<Revisioned<Vec<AutomationView>>, ApiError> {
        let target = origin(&self.configuration.read(), id)
            .ok_or(ApiError::NotFound(Automation::UNKNOWN))?;
        let (_, snapshot) = self
            .configuration
            .update(&target, revision, |document| {
                let index =
                    position(document, id).ok_or(ApiError::NotFound(Automation::UNKNOWN))?;
                remove(document, index);
                Ok(())
            })
            .await?;
        self.views.sink.cache.refresh(&snapshot.document);
        Ok(Revisioned::new(
            self.views.automations(&snapshot.document),
            snapshot.revision,
        ))
    }
}
