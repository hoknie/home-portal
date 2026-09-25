use portal_model::TlsPolicy;
use serde::Deserialize;

use crate::types::ProxyChoice;

#[derive(Debug, Deserialize)]
pub struct ProxySettingsRequest {
    pub enabled: bool,
    #[serde(default)]
    pub http_port: Option<i64>,
    #[serde(default)]
    pub https_port: Option<i64>,
    #[serde(default)]
    pub portal_host: Option<String>,
    #[serde(default)]
    pub cookie_domain: Option<String>,
    #[serde(default)]
    pub tls: TlsPolicy,
}

impl ProxySettingsRequest {
    pub fn into_choice(self) -> ProxyChoice {
        let blank_to_none = |text: Option<String>| {
            text.map(|text| text.trim().to_ascii_lowercase())
                .filter(|text| !text.is_empty())
        };
        ProxyChoice {
            enabled: self.enabled,
            http_port: self.http_port,
            https_port: self.https_port,
            portal_host: blank_to_none(self.portal_host),
            cookie_domain: blank_to_none(self.cookie_domain),
            tls: self.tls,
        }
    }
}
