use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::ApiError;

use super::checked;
use crate::repositories::{replace_workflow, workflow_origin, workflow_position};
use crate::services::{Views, decode_workflow};
use crate::types::{RawWorkflow, Workflow, WorkflowView};

#[derive(Clone)]
pub struct ChangeWorkflow {
    configuration: Arc<ConfigStore>,
    views: Views,
}

impl ChangeWorkflow {
    pub fn new(configuration: Arc<ConfigStore>, views: Views) -> ChangeWorkflow {
        ChangeWorkflow {
            configuration,
            views,
        }
    }

    pub async fn run(
        &self,
        id: &str,
        raw: RawWorkflow,
        revision: &Revision,
    ) -> Result<Revisioned<WorkflowView>, ApiError> {
        let workflow = decode_workflow(&raw).map_err(ApiError::Invalid)?;
        let target = workflow_origin(&self.configuration.read(), id)
            .ok_or(ApiError::NotFound(Workflow::UNKNOWN))?;
        let (_, snapshot) = self
            .configuration
            .update(&target, revision, |document| {
                let index =
                    workflow_position(document, id).ok_or(ApiError::NotFound(Workflow::UNKNOWN))?;
                if workflow.id != id && workflow_position(document, &workflow.id).is_some() {
                    return Err(ApiError::invalid("id", Workflow::TAKEN_ID));
                }
                replace_workflow(document, index, &workflow, &raw.steps);
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
