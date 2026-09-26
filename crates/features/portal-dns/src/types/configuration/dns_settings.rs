use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;

use portal_model::{Environment, TlsPolicy};

use super::DnsRecord;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsSettings {
    pub enabled: bool,
    pub address: IpAddr,
    pub port: u16,
    pub zones: Vec<String>,
    pub ttl: u32,
    pub addresses: BTreeMap<Environment, ProxyAddresses>,
    pub records: Vec<DnsRecord>,
    pub tls: DnsTls,
    pub https: DnsHttps,
    pub proxy: ProxyView,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProxyAddresses {
    pub v4: Option<Ipv4Addr>,
    pub v6: Option<Ipv6Addr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsTls {
    pub enabled: bool,
    pub port: u16,
    pub certificate: Option<PathBuf>,
    pub key: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DnsHttps {
    pub enabled: bool,
    pub host: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProxyView {
    pub enabled: bool,
    pub managed: bool,
    pub portal_host: Option<String>,
    pub https_port: Option<u16>,
    pub tls: TlsPolicy,
}

impl DnsSettings {
    pub const DEFAULT_PORT: u16 = 53;
    pub const DEFAULT_TLS_PORT: u16 = 853;
    pub const DEFAULT_TTL: u32 = 60;
    pub const SHORTEST_TTL: u32 = 5;
    pub const LONGEST_TTL: u32 = 86_400;

    pub const DOH_PATH: &'static str = "/dns-query";
    pub const HTTPS_PORT: u16 = 443;

    pub fn doh_url(&self) -> Option<String> {
        let host = self.secure_host()?;
        let port = self
            .proxy
            .https_port
            .filter(|port| *port != Self::HTTPS_PORT);
        Some(match port {
            Some(port) => format!("https://{host}:{port}{}", Self::DOH_PATH),
            None => format!("https://{host}{}", Self::DOH_PATH),
        })
    }

    pub fn secure_host(&self) -> Option<&str> {
        self.https
            .host
            .as_deref()
            .or(self.proxy.portal_host.as_deref())
    }
}

impl Default for DnsSettings {
    fn default() -> DnsSettings {
        DnsSettings {
            enabled: false,
            address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port: Self::DEFAULT_PORT,
            zones: Vec::new(),
            ttl: Self::DEFAULT_TTL,
            addresses: BTreeMap::new(),
            records: Vec::new(),
            tls: DnsTls {
                enabled: false,
                port: Self::DEFAULT_TLS_PORT,
                certificate: None,
                key: None,
            },
            https: DnsHttps::default(),
            proxy: ProxyView::default(),
        }
    }
}
