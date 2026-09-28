use portal_feature::EventName;

use crate::types::{FilterDescription, KindDescription, Operator};

#[derive(Debug, Clone)]
pub struct WorkflowCatalogueView {
    pub kinds: Vec<KindDescription>,
    pub operators: Vec<Operator>,
    pub events: Vec<(EventName, Vec<&'static str>)>,
    pub filters: Vec<FilterDescription>,
    pub operations: Vec<FilterDescription>,
}
