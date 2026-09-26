use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct RawDnsView {
    pub enabled: Option<bool>,
    #[serde(default)]
    pub https: RawDnsHttpsView,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawDnsHttpsView {
    pub enabled: Option<bool>,
    pub host: Option<String>,
}
