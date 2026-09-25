use serde::{Deserialize, Serialize};

use super::SecretResponse;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretsResponse {
    pub secrets: Vec<SecretResponse>,
}
