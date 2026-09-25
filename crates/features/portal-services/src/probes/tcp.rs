use std::time::{Duration, Instant};

use portal_model::{Diagnosis, ProbeOutcome, ServiceState};
use tokio::net::TcpStream;

use crate::helpers::{classify_io, probe_host, tcp_port};
use crate::types::ServiceEntry;

pub struct TcpProbe;

impl TcpProbe {
    pub async fn probe(entry: &ServiceEntry, address: &str) -> ProbeOutcome {
        let target = probe_host(address)
            .and_then(|host| tcp_port(address, entry.probe.port).map(|port| (host, port)));
        let (host, port) = match target {
            Ok(target) => target,
            Err(message) => {
                return ProbeOutcome::failed(ServiceState::Unreadable, None, message);
            }
        };
        let started = Instant::now();
        let limit = Duration::from_secs(entry.probe.timeout_seconds);
        let connected =
            tokio::time::timeout(limit, TcpStream::connect((host.as_str(), port))).await;
        let latency = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        match connected {
            Err(_) => ProbeOutcome::failed(ServiceState::Down, None, "timed out".to_string())
                .because(Diagnosis::Timeout),
            Ok(Err(error)) => classify_io(&error).into_outcome(None),
            Ok(Ok(stream)) => {
                drop(stream);
                let state = if latency > entry.probe.degraded_after_milliseconds {
                    ServiceState::Degraded
                } else {
                    ServiceState::Up
                };
                ProbeOutcome::answered(state, latency)
            }
        }
    }
}
