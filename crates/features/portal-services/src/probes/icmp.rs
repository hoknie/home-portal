use std::io::{self, ErrorKind};
use std::net::{IpAddr, SocketAddr, UdpSocket as StandardSocket};
use std::time::{Duration, Instant};

use portal_model::{Diagnosis, ProbeOutcome, ServiceState};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::UdpSocket;

use crate::helpers::{TOKEN_LENGTH, classify_io, echo_request, is_echo_reply, probe_host};
use crate::types::ServiceEntry;

pub struct IcmpProbe;

impl IcmpProbe {
    pub const NOT_PERMITTED: &'static str = "this process may not send ICMP echo requests";
    pub const REPLY_BUFFER: usize = 1500;

    pub async fn probe(entry: &ServiceEntry, address: &str) -> ProbeOutcome {
        let limit = Duration::from_secs(entry.probe.timeout_seconds);
        let started = Instant::now();
        let target = match Self::resolve(address).await {
            Ok(target) => target,
            Err(outcome) => return *outcome,
        };
        let answered = tokio::time::timeout(limit, Self::echo(target)).await;
        let latency = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        match answered {
            Err(_) => ProbeOutcome::failed(ServiceState::Down, None, "no echo reply".to_string())
                .because(Diagnosis::Timeout),
            Ok(Err(error)) if Self::forbidden(&error) => ProbeOutcome::failed(
                ServiceState::Unreadable,
                None,
                format!("{}: {error}", Self::NOT_PERMITTED),
            )
            .because(Diagnosis::IcmpNotPermitted),
            Ok(Err(error)) => classify_io(&error).into_outcome(None),
            Ok(Ok(())) => {
                let state = if latency > entry.probe.degraded_after_milliseconds {
                    ServiceState::Degraded
                } else {
                    ServiceState::Up
                };
                ProbeOutcome::answered(state, latency)
            }
        }
    }

    async fn resolve(address: &str) -> Result<IpAddr, Box<ProbeOutcome>> {
        let host = probe_host(address).map_err(|message| {
            Box::new(ProbeOutcome::failed(
                ServiceState::Unreadable,
                None,
                message,
            ))
        })?;
        if let Ok(ip) = host.parse::<IpAddr>() {
            return Ok(ip);
        }
        let found = tokio::net::lookup_host((host.as_str(), 0))
            .await
            .map_err(|error| {
                Box::new(
                    ProbeOutcome::failed(ServiceState::Unreadable, None, error.to_string())
                        .because(Diagnosis::NameNotResolved),
                )
            })?
            .next();
        found.map(|socket| socket.ip()).ok_or_else(|| {
            Box::new(
                ProbeOutcome::failed(
                    ServiceState::Unreadable,
                    None,
                    format!("{host} has no address"),
                )
                .because(Diagnosis::NameNotResolved),
            )
        })
    }

    async fn echo(target: IpAddr) -> io::Result<()> {
        let (domain, protocol) = if target.is_ipv4() {
            (Domain::IPV4, Protocol::ICMPV4)
        } else {
            (Domain::IPV6, Protocol::ICMPV6)
        };
        let socket = Socket::new(domain, Type::DGRAM, Some(protocol))?;
        socket.set_nonblocking(true)?;
        let socket = UdpSocket::from_std(StandardSocket::from(socket))?;
        let sequence = Self::sequence();
        let token: [u8; TOKEN_LENGTH] = u64::from(sequence)
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .to_be_bytes();
        socket
            .send_to(
                &echo_request(target, sequence, token),
                SocketAddr::new(target, 0),
            )
            .await?;
        let mut buffer = vec![0u8; Self::REPLY_BUFFER];
        loop {
            let (length, from) = socket.recv_from(&mut buffer).await?;
            if from.ip() == target && is_echo_reply(&buffer[..length], sequence, token) {
                return Ok(());
            }
        }
    }

    fn sequence() -> u16 {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.subsec_nanos())
            .unwrap_or_default();
        u16::try_from((nanos ^ (nanos >> 16)) & 0xffff).unwrap_or_default()
    }

    fn forbidden(error: &io::Error) -> bool {
        matches!(error.kind(), ErrorKind::PermissionDenied)
            || matches!(
                error.raw_os_error(),
                Some(1) | Some(13) | Some(43) | Some(93)
            )
    }
}
