use std::collections::BTreeMap;

use super::{DnsHttpsChoice, DnsTlsChoice};

#[derive(Debug, Clone, Default)]
pub struct DnsChoice {
    pub enabled: bool,
    pub address: String,
    pub port: u16,
    pub zones: Vec<String>,
    pub ttl: u32,
    pub addresses: BTreeMap<String, Vec<String>>,
    pub tls: DnsTlsChoice,
    pub https: DnsHttpsChoice,
}
