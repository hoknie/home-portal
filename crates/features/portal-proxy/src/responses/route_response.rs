use portal_model::TlsMode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RouteResponse {
    pub host: String,
    pub address: String,
    pub service: Option<String>,
    pub upstream: String,
    pub tls: TlsMode,
    pub auth: Vec<String>,
    pub environments: Vec<String>,
}
