use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::ApiError;

use crate::repositories::{remove_workflow, workflow_origin, workflow_position};
use crate::services::{Views, users_of};
use crate::types::{Workflow, WorkflowView};

#[derive(Clone)]
pub struct DeleteWorkflow {
    configuration: Arc<ConfigStore>,
    views: Views,
}

impl DeleteWorkflow {
    pub const IN_USE: &'static str = "the workflow is used by";

    pub fn new(configuration: Arc<ConfigStore>, views: Views) -> DeleteWorkflow {
        DeleteWorkflow {
            configuration,
            views,
        }
    }

    pub async fn run(
        &self,
        id: &str,
        revision: &Revision,
    ) -> Result<Revisioned<Vec<WorkflowView>>, ApiError> {
        let target = workflow_origin(&self.configuration.read(), id)
            .ok_or(ApiError::NotFound(Workflow::UNKNOWN))?;
        let cache = &self.views.sink.cache;
        let users = users_of(
            id,
            (&cache.automations(), &cache.webhooks(), &cache.workflows()),
        );
        if !users.is_empty() {
            let names: Vec<String> = users.into_iter().map(|usage| usage.id).collect();
            return Err(ApiError::Conflict(format!(
                "{} {}",
                Self::IN_USE,
                names.join(", ")
            )));
        }
        let (_, snapshot) = self
            .configuration
            .update(&target, revision, |document| {
                let index =
                    workflow_position(document, id).ok_or(ApiError::NotFound(Workflow::UNKNOWN))?;
                remove_workflow(document, index);
                Ok(())
            })
            .await?;
        self.views.sink.cache.refresh(&snapshot.document);
        Ok(Revisioned::new(
            self.views.workflows(&snapshot.document),
            snapshot.revision,
        ))
    }
}
