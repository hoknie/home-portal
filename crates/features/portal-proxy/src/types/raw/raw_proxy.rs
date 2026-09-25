use portal_model::TlsPolicy;
use serde::Deserialize;

use super::RawCaddy;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct RawProxy {
    pub enabled: Option<bool>,
    pub managed: Option<bool>,
    pub http_port: Option<i64>,
    pub https_port: Option<i64>,
    pub admin: Option<String>,
    pub portal_host: Option<String>,
    pub cookie_domain: Option<String>,
    pub tls: Option<TlsPolicy>,
    pub caddy: Option<RawCaddy>,
}
