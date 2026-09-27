use std::net::{IpAddr, SocketAddr};

use axum::http::HeaderMap;
use portal_auth::{Connection, CookieScope};
use portal_model::Publication;
use portal_network::{CurrentNetwork, NetworkSettings, client_address};
use portal_proxy::{CurrentProxySettings, TrustedPeers};

pub struct NetworkConnection {
    pub network: CurrentNetwork,
    pub proxy: CurrentProxySettings,
}

impl NetworkConnection {
    pub const FORWARDED_HOST: &'static str = "x-forwarded-host";

    fn proxied_domain(
        &self,
        network: &NetworkSettings,
        peer: Option<SocketAddr>,
        headers: &HeaderMap,
    ) -> Option<String> {
        let settings = self.proxy.run().ok()?;
        let domain = settings.cookie_domain()?;
        let peer = peer?.ip();
        if !network
            .trusted_proxies
            .iter()
            .any(|trusted| trusted.contains(&peer))
        {
            return None;
        }
        let forwarded = headers.get(Self::FORWARDED_HOST)?.to_str().ok()?.trim();
        let host = match forwarded.rsplit_once(':') {
            Some((name, port)) if port.chars().all(|character| character.is_ascii_digit()) => name,
            _ => forwarded,
        }
        .to_ascii_lowercase();
        Publication::is_within(&host, domain).then(|| domain.to_string())
    }
}

impl Connection for NetworkConnection {
    fn client_address(&self, peer: Option<SocketAddr>, headers: &HeaderMap) -> IpAddr {
        let settings = self.network.run().settings.unwrap_or_default();
        client_address(peer, headers, &settings.trusted_proxies)
    }

    fn cookie_scope(&self, peer: Option<SocketAddr>, headers: &HeaderMap) -> CookieScope {
        let network = self.network.run().settings.unwrap_or_default();
        match self.proxied_domain(&network, peer, headers) {
            Some(domain) => CookieScope {
                secure: true,
                domain: Some(domain),
            },
            None => CookieScope {
                secure: network.secure(),
                domain: None,
            },
        }
    }
}

impl TrustedPeers for NetworkConnection {
    fn trusts(&self, peer: IpAddr) -> bool {
        self.network
            .run()
            .settings
            .map(|settings| {
                settings
                    .trusted_proxies
                    .iter()
                    .any(|network| network.contains(&peer))
            })
            .unwrap_or(false)
    }
}
