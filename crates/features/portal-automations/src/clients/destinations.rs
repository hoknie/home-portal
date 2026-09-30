use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use reqwest::Url;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Destinations {
    pub allow_loopback: bool,
}

impl Destinations {
    pub const LOOPBACK: &'static str = "a loopback address";
    pub const LINK_LOCAL: &'static str = "a link-local address";
    pub const UNSPECIFIED: &'static str = "an unspecified address";

    pub fn guarded() -> Destinations {
        Destinations {
            allow_loopback: false,
        }
    }

    pub fn refusal(&self, address: IpAddr) -> Option<&'static str> {
        let address = address.to_canonical();
        if address.is_unspecified() {
            return Some(Self::UNSPECIFIED);
        }
        if address.is_loopback() && !self.allow_loopback {
            return Some(Self::LOOPBACK);
        }
        let link_local = match address {
            IpAddr::V4(v4) => v4.is_link_local(),
            IpAddr::V6(v6) => (v6.segments()[0] & 0xffc0) == 0xfe80,
        };
        link_local.then_some(Self::LINK_LOCAL)
    }

    pub fn url_problem(&self, url: &Url) -> Option<String> {
        let host = url.host_str()?;
        let literal = host.trim_start_matches('[').trim_end_matches(']');
        let address: IpAddr = literal.parse().ok()?;
        self.refusal(address)
            .map(|reason| format!("{host} is {reason}, which a workflow may not reach"))
    }

    pub fn resolver(self) -> Arc<GuardedResolver> {
        Arc::new(GuardedResolver { destinations: self })
    }
}

pub struct GuardedResolver {
    destinations: Destinations,
}

impl Resolve for GuardedResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let destinations = self.destinations;
        let host = name.as_str().to_string();
        Box::pin(async move {
            let found: Vec<SocketAddr> =
                tokio::net::lookup_host((host.as_str(), 0)).await?.collect();
            let refused = found
                .iter()
                .find_map(|address| destinations.refusal(address.ip()));
            match refused {
                Some(reason) => {
                    Err(format!("{host} is {reason}, which a workflow may not reach").into())
                }
                None => Ok(Box::new(found.into_iter()) as Addrs),
            }
        })
    }
}
