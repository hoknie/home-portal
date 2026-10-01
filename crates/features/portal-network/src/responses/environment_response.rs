use portal_model::Environment;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EnvironmentResponse {
    pub environment: Environment,
    pub detected: Environment,
    pub switchable: bool,
    pub environments: Vec<Environment>,
}
