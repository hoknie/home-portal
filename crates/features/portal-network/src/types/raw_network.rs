use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct RawNetwork {
    pub address: Option<String>,
    pub port: Option<i64>,
    pub public_url: Option<String>,
    pub trusted_proxies: Option<Vec<String>>,
}
