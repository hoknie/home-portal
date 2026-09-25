use portal_model::{Publication, TlsPolicy};

use super::{AdminAddress, CaddySource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxySettings {
    pub enabled: bool,
    pub managed: bool,
    pub http_port: u16,
    pub https_port: u16,
    pub admin: AdminAddress,
    pub portal_host: Option<String>,
    pub cookie_domain: Option<String>,
    pub tls: TlsPolicy,
    pub caddy: CaddySource,
}

impl Default for ProxySettings {
    fn default() -> ProxySettings {
        ProxySettings {
            enabled: false,
            managed: false,
            http_port: Publication::HTTP_PORT,
            https_port: Publication::HTTPS_PORT,
            admin: AdminAddress::default(),
            portal_host: None,
            cookie_domain: None,
            tls: TlsPolicy::default(),
            caddy: CaddySource::default(),
        }
    }
}

impl ProxySettings {
    pub fn origin(&self, host: &str) -> String {
        Publication::origin(host, self.https_port)
    }

    pub fn active(&self) -> Option<&str> {
        self.portal_host.as_deref().filter(|_| self.enabled)
    }

    pub fn cookie_domain(&self) -> Option<&str> {
        self.cookie_domain.as_deref().filter(|_| self.enabled)
    }

    pub fn tls_of(&self, publication: &Publication) -> TlsPolicy {
        publication.tls.clone().unwrap_or_else(|| self.tls.clone())
    }
}
