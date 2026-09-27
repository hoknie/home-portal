use std::collections::BTreeMap;

use serde::Deserialize;

use crate::types::{DnsChoice, DnsHttpsChoice, DnsTlsChoice};

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

impl DnsRequest {
    pub fn into_choice(self) -> DnsChoice {
        DnsChoice {
            enabled: self.enabled,
            address: self.address,
            port: self.port,
            zones: self.zones,
            ttl: self.ttl,
            addresses: self.addresses,
            tls: DnsTlsChoice {
                enabled: self.tls.enabled,
                port: self.tls.port,
                certificate: self.tls.certificate,
                key: self.tls.key,
            },
            https: DnsHttpsChoice {
                enabled: self.https.enabled,
                host: self.https.host,
            },
        }
    }
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
