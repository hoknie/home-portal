use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned, Section};
use portal_feature::ApiError;

use crate::repositories::{append, position};
use crate::services::{AutomationSink, automation_problems};
use crate::types::{Automation, AutomationView, RawAutomation};

#[derive(Clone)]
pub struct CreateAutomation {
    configuration: Arc<ConfigStore>,
    sink: Arc<AutomationSink>,
}

impl CreateAutomation {
    pub fn new(configuration: Arc<ConfigStore>, sink: Arc<AutomationSink>) -> CreateAutomation {
        CreateAutomation {
            configuration,
            sink,
        }
    }

    pub async fn run(
        &self,
        raw: &RawAutomation,
        revision: &Revision,
    ) -> Result<Revisioned<AutomationView>, ApiError> {
        let automation = Automation::decode(raw).map_err(ApiError::Invalid)?;
        let problems = automation_problems(&self.configuration.read().document, &automation);
        if !problems.is_empty() {
            return Err(ApiError::Invalid(problems));
        }
        let target = self.configuration.home_of(Section::Automations);
        let (_, snapshot) = self
            .configuration
            .update(&target, revision, |document| {
                if position(document, &automation.id).is_some() {
                    return Err(ApiError::invalid("id", Automation::TAKEN_ID));
                }
                append(document, &automation);
                Ok(())
            })
            .await?;
        self.sink.cache.refresh(&snapshot.document);
        let view = AutomationView {
            automation,
            last: None,
            active: None,
        };
        Ok(Revisioned::new(view, snapshot.revision))
    }
}
