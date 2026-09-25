use serde::Deserialize;

use crate::types::RawNetwork;

#[derive(Debug, Deserialize)]
pub struct NetworkRequest {
    pub address: String,
    pub port: i64,
    #[serde(default)]
    pub public_url: Option<String>,
    #[serde(default)]
    pub trusted_proxies: Vec<String>,
}

impl NetworkRequest {
    pub fn into_raw(self) -> RawNetwork {
        RawNetwork {
            address: Some(self.address),
            port: Some(self.port),
            public_url: self.public_url,
            trusted_proxies: Some(self.trusted_proxies),
        }
    }
}
