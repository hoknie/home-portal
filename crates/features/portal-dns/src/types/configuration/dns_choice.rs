use std::collections::BTreeMap;

use super::{DnsHttpsChoice, DnsTlsChoice};

#[derive(Debug, Clone, Default)]
pub struct DnsChoice {
    pub address: String,
    pub port: u16,
    pub zones: Vec<String>,
    pub ttl: u32,
    pub addresses: BTreeMap<String, Vec<String>>,
    pub tls: DnsTlsChoice,
    pub https: DnsHttpsChoice,
}

impl DnsChoice {
    pub fn normalized(&self) -> DnsChoice {
        let trimmed = |values: &[String]| -> Vec<String> {
            values.iter().map(|text| text.trim().to_string()).collect()
        };
        let present = |text: &Option<String>| {
            text.as_deref()
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(String::from)
        };
        DnsChoice {
            address: self.address.trim().to_string(),
            port: self.port,
            zones: trimmed(&self.zones),
            ttl: self.ttl,
            addresses: self
                .addresses
                .iter()
                .map(|(environment, found)| (environment.clone(), trimmed(found)))
                .collect(),
            tls: DnsTlsChoice {
                enabled: self.tls.enabled,
                port: self.tls.port,
                certificate: present(&self.tls.certificate),
                key: present(&self.tls.key),
            },
            https: DnsHttpsChoice {
                enabled: self.https.enabled,
                host: present(&self.https.host),
            },
        }
    }
}
