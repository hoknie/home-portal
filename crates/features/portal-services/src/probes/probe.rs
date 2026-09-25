use std::net::IpAddr;
use std::time::Instant;

use ipnet::IpNet;
use portal_model::{Diagnosis, ProbeOutcome};

use super::{HttpProbe, IcmpProbe, TcpProbe};
use crate::helpers::{
    PLATFORM_REFUSES_LOCAL_NETWORK, local_network_denied, probe_host, refine_unreachable,
};
use crate::types::{Attempt, ProbeKind, ServiceEntry};

pub struct Probe {
    http: HttpProbe,
    subnets: fn() -> Vec<IpNet>,
}

impl Probe {
    pub fn new() -> Result<Probe, String> {
        Ok(Probe {
            http: HttpProbe::new()?,
            subnets: Self::host_subnets,
        })
    }

    pub async fn run(&self, entry: &ServiceEntry, address: &str) -> ProbeOutcome {
        let started = Instant::now();
        let outcome = match entry.probe.kind {
            ProbeKind::Http => self.http.probe(entry, address).await,
            ProbeKind::Tcp => TcpProbe::probe(entry, address).await,
            ProbeKind::Icmp => IcmpProbe::probe(entry, address).await,
        };
        if outcome.diagnosis != Some(Diagnosis::HostUnreachable) {
            return outcome;
        }
        let attempt = Attempt {
            target: Self::target_address(address).await,
            elapsed: started.elapsed(),
        };
        let denied =
            local_network_denied(&attempt, &(self.subnets)(), PLATFORM_REFUSES_LOCAL_NETWORK);
        refine_unreachable(outcome, denied)
    }

    pub fn host_subnets() -> Vec<IpNet> {
        if_addrs::get_if_addrs()
            .unwrap_or_default()
            .into_iter()
            .filter(|interface| !interface.is_loopback())
            .filter_map(|interface| match interface.addr {
                if_addrs::IfAddr::V4(v4) => {
                    IpNet::with_netmask(IpAddr::V4(v4.ip), IpAddr::V4(v4.netmask)).ok()
                }
                if_addrs::IfAddr::V6(v6) => {
                    IpNet::with_netmask(IpAddr::V6(v6.ip), IpAddr::V6(v6.netmask)).ok()
                }
            })
            .collect()
    }

    async fn target_address(address: &str) -> Option<IpAddr> {
        let host = probe_host(address).ok()?;
        if let Ok(ip) = host.parse::<IpAddr>() {
            return Some(ip);
        }
        tokio::net::lookup_host((host.as_str(), 0))
            .await
            .ok()?
            .next()
            .map(|socket| socket.ip())
    }
}
