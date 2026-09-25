use portal_model::TlsMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteResponse {
    pub host: String,
    pub address: String,
    pub service: Option<String>,
    pub upstream: String,
    pub tls: TlsMode,
    pub auth: Vec<String>,
    pub environments: Vec<String>,
}
