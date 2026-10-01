use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ServiceResponse;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ServicesResponse {
    pub services: Vec<ServiceResponse>,
}
