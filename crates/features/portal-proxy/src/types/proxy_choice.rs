use portal_model::TlsPolicy;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProxyChoice {
    pub enabled: bool,
    pub http_port: Option<i64>,
    pub https_port: Option<i64>,
    pub portal_host: Option<String>,
    pub cookie_domain: Option<String>,
    pub tls: TlsPolicy,
}
