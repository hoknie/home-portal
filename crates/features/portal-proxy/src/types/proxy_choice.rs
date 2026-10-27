use portal_model::TlsPolicy;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProxyChoice {
    pub http_port: Option<i64>,
    pub https_port: Option<i64>,
    pub portal_host: Option<String>,
    pub cookie_domain: Option<String>,
    pub tls: TlsPolicy,
}

impl ProxyChoice {
    pub fn normalized(&self) -> ProxyChoice {
        ProxyChoice {
            tls: self.tls.normalized(),
            ..self.clone()
        }
    }
}
