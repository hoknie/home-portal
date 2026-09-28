use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::ApiError;

use super::checked;
use crate::repositories::{append_workflow, workflow_position};
use crate::services::{Views, decode_workflow};
use crate::types::{RawWorkflow, Workflow, WorkflowView};

#[derive(Clone)]
pub struct CreateWorkflow {
    configuration: Arc<ConfigStore>,
    views: Views,
}

impl CreateWorkflow {
    pub fn new(configuration: Arc<ConfigStore>, views: Views) -> CreateWorkflow {
        CreateWorkflow {
            configuration,
            views,
        }
    }

    pub async fn run(
        &self,
        raw: RawWorkflow,
        revision: &Revision,
    ) -> Result<Revisioned<WorkflowView>, ApiError> {
        let workflow = decode_workflow(&raw).map_err(ApiError::Invalid)?;
        let target = self.configuration.writes_to();
        let (_, snapshot) = self
            .configuration
            .update(&target, revision, |document| {
                if workflow_position(document, &workflow.id).is_some() {
                    return Err(ApiError::invalid("id", Workflow::TAKEN_ID));
                }
                append_workflow(document, &workflow, &raw.steps);
                checked(document, &workflow.id)
            })
            .await?;
        self.views.sink.cache.refresh(&snapshot.document);
        let view = self
            .views
            .workflows(&snapshot.document)
            .into_iter()
            .find(|view| view.workflow.id == workflow.id)
            .ok_or(ApiError::NotFound(Workflow::UNKNOWN))?;
        Ok(Revisioned::new(view, snapshot.revision))
    }
}
