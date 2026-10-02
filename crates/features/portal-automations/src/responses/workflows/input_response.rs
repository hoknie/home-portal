use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;

use crate::types::{InputDeclaration, OutputDeclaration};

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct InputResponse {
    pub name: String,
    #[serde(rename = "type")]
    pub input_type: String,
    pub default: Option<Value>,
    pub description: Option<String>,
}

impl InputResponse {
    pub fn of(input: &InputDeclaration) -> InputResponse {
        InputResponse {
            name: input.name.clone(),
            input_type: input.input_type.name().to_string(),
            default: input.default.clone(),
            description: input.description.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct WorkflowOutputResponse {
    pub name: String,
    pub value: String,
    pub description: Option<String>,
}

impl WorkflowOutputResponse {
    pub fn of(output: &OutputDeclaration) -> WorkflowOutputResponse {
        WorkflowOutputResponse {
            name: output.name.clone(),
            value: output.value.clone(),
            description: output.description.clone(),
        }
    }
}
