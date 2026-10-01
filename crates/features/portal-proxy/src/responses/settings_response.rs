use portal_model::TlsPolicy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SettingsResponse {
    pub http_port: u16,
    pub https_port: u16,
    pub portal_host: Option<String>,
    pub cookie_domain: Option<String>,
    pub tls: TlsPolicy,
}
