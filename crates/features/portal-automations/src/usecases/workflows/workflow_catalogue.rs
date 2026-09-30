use portal_feature::EventName;

use crate::types::{Catalogue, FILTERS, KINDS, OPERATIONS, Operator, WorkflowCatalogueView};

#[derive(Clone, Default)]
pub struct WorkflowCatalogue;

impl WorkflowCatalogue {
    pub fn run(&self) -> WorkflowCatalogueView {
        WorkflowCatalogueView {
            kinds: KINDS.to_vec(),
            operators: Operator::ALL.to_vec(),
            events: EventName::ALL
                .iter()
                .map(|event| (*event, Catalogue::carried_by(*event)))
                .collect(),
            filters: FILTERS.to_vec(),
            operations: OPERATIONS.to_vec(),
        }
    }
}
