use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use crate::types::WorkflowCall;

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowCallResponse {
    pub id: String,
    pub inputs: BTreeMap<String, Value>,
}

impl WorkflowCallResponse {
    pub fn of(call: &WorkflowCall) -> WorkflowCallResponse {
        WorkflowCallResponse {
            id: call.id.clone(),
            inputs: call
                .inputs
                .iter()
                .map(|(name, value)| (name.clone(), value.as_json()))
                .collect(),
        }
    }
}
