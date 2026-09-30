use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::ApiError;

use crate::repositories::{origin, position, replace};
use crate::services::{Views, automation_problems};
use crate::types::{Automation, AutomationView, RawAutomation};

#[derive(Clone)]
pub struct ChangeAutomation {
    configuration: Arc<ConfigStore>,
    views: Views,
}

impl ChangeAutomation {
    pub fn new(configuration: Arc<ConfigStore>, views: Views) -> ChangeAutomation {
        ChangeAutomation {
            configuration,
            views,
        }
    }

    pub async fn run(
        &self,
        id: &str,
        raw: &RawAutomation,
        revision: &Revision,
    ) -> Result<Revisioned<AutomationView>, ApiError> {
        let automation = Automation::decode(raw).map_err(ApiError::Invalid)?;
        let problems = automation_problems(&self.configuration.read().document, &automation);
        if !problems.is_empty() {
            return Err(ApiError::Invalid(problems));
        }
        let target = origin(&self.configuration.read(), id)
            .ok_or(ApiError::NotFound(Automation::UNKNOWN))?;
        let (_, snapshot) = self
            .configuration
            .update(&target, revision, |document| {
                let index =
                    position(document, id).ok_or(ApiError::NotFound(Automation::UNKNOWN))?;
                if automation.id != id && position(document, &automation.id).is_some() {
                    return Err(ApiError::invalid("id", Automation::TAKEN_ID));
                }
                replace(document, index, &automation);
                Ok(())
            })
            .await?;
        self.views.sink.cache.refresh(&snapshot.document);
        Ok(Revisioned::new(
            self.views.automation(automation),
            snapshot.revision,
        ))
    }
}
