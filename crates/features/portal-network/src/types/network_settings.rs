use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use ipnet::IpNet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub address: IpAddr,
    pub port: u16,
    pub public_url: Option<String>,
    pub trusted_proxies: Vec<IpNet>,
}

impl NetworkSettings {
    pub const DEFAULT_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
    pub const DEFAULT_PORT: u16 = 8080;

    pub fn socket_address(&self) -> SocketAddr {
        SocketAddr::new(self.address, self.port)
    }

    pub fn secure(&self) -> bool {
        self.public_url
            .as_deref()
            .is_some_and(|url| url.starts_with("https://"))
    }
}

impl Default for NetworkSettings {
    fn default() -> NetworkSettings {
        NetworkSettings {
            address: Self::DEFAULT_ADDRESS,
            port: Self::DEFAULT_PORT,
            public_url: None,
            trusted_proxies: Vec::new(),
        }
    }
}
