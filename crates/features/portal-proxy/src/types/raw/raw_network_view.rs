use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct RawNetworkView {
    pub public_url: Option<String>,
    pub trusted_proxies: Option<Vec<String>>,
}
