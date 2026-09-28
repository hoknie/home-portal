use serde::Serialize;

use super::{EventFieldsResponse, FilterResponse, OperatorResponse, StepKindResponse};
use crate::types::WorkflowCatalogueView;

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowCatalogueResponse {
    pub kinds: Vec<StepKindResponse>,
    pub operators: Vec<OperatorResponse>,
    pub events: Vec<EventFieldsResponse>,
    pub filters: Vec<FilterResponse>,
    pub operations: Vec<FilterResponse>,
}

impl WorkflowCatalogueResponse {
    pub fn of(view: &WorkflowCatalogueView) -> WorkflowCatalogueResponse {
        WorkflowCatalogueResponse {
            kinds: view.kinds.iter().map(StepKindResponse::of).collect(),
            operators: view
                .operators
                .iter()
                .copied()
                .map(OperatorResponse::of)
                .collect(),
            events: view
                .events
                .iter()
                .map(|(event, fields)| EventFieldsResponse {
                    name: event.name().to_string(),
                    fields: fields.iter().map(|field| field.to_string()).collect(),
                })
                .collect(),
            filters: view.filters.iter().map(FilterResponse::of).collect(),
            operations: view.operations.iter().map(FilterResponse::of).collect(),
        }
    }
}
