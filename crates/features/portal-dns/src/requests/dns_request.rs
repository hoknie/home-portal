use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DnsRequest {
    pub enabled: bool,
    pub address: String,
    pub port: u16,
    #[serde(default)]
    pub zones: Vec<String>,
    pub ttl: u32,
    #[serde(default)]
    pub addresses: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub tls: DnsTlsRequest,
    #[serde(default)]
    pub https: DnsHttpsRequest,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DnsTlsRequest {
    pub enabled: bool,
    pub port: Option<u16>,
    pub certificate: Option<String>,
    pub key: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DnsHttpsRequest {
    pub enabled: bool,
    pub host: Option<String>,
}
