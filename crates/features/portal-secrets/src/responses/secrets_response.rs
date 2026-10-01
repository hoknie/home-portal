use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::SecretResponse;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SecretsResponse {
    pub secrets: Vec<SecretResponse>,
}
