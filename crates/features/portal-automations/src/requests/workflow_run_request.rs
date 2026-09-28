use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WorkflowRunRequest {
    #[serde(default)]
    pub inputs: BTreeMap<String, Value>,
}

impl WorkflowRunRequest {
    pub fn parse(body: &[u8]) -> Result<WorkflowRunRequest, String> {
        if body.iter().all(u8::is_ascii_whitespace) {
            return Ok(WorkflowRunRequest::default());
        }
        serde_json::from_slice(body).map_err(|error| error.to_string())
    }
}
