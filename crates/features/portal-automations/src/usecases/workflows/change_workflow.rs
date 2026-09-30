use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::{ApiError, Rights};

use super::checked;
use crate::repositories::{replace_workflow, workflow_origin, workflow_position};
use crate::services::{Views, decode_workflow, secrets_allowed};
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
        (revision, rights): (&Revision, &Rights),
    ) -> Result<Revisioned<WorkflowView>, ApiError> {
        let workflow = decode_workflow(&raw).map_err(ApiError::Invalid)?;
        secrets_allowed(&workflow, rights)?;
        let current = self.configuration.read();
        let target = workflow_origin(&current, id).ok_or(ApiError::NotFound(Workflow::UNKNOWN))?;
        let renamed = workflow.id != id;
        if renamed && workflow_origin(&current, &workflow.id).is_some() {
            return Err(ApiError::invalid("id", Workflow::TAKEN_ID));
        }
        let edit = |document: &mut toml_edit::DocumentMut| {
            let index =
                workflow_position(document, id).ok_or(ApiError::NotFound(Workflow::UNKNOWN))?;
            replace_workflow(document, index, &workflow, &raw.steps);
            checked(document, &workflow.id)
        };
        let (_, snapshot) = if renamed && self.configuration.in_workflow_folder(&target) {
            let moved = self.configuration.workflow_file(&workflow.id);
            if moved.exists() {
                return Err(ApiError::invalid("id", Workflow::TAKEN_ID));
            }
            self.configuration
                .update_moved(&target, &moved, revision, edit)
                .await?
        } else {
            self.configuration.update(&target, revision, edit).await?
        };
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
