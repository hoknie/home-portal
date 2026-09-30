use std::io::{self, ErrorKind};
use std::net::{Ipv4Addr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::ports::PermissionCheck;
use crate::types::{Advice, Finding, PermissionCode};

pub const MDNS_GROUP: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
pub const MDNS_PORT: u16 = 5353;
pub const QUESTION: [&str; 3] = ["_home-portal", "_tcp", "local"];
pub const POINTER_TYPE: u16 = 12;
pub const INTERNET_CLASS: u16 = 1;

pub type Sender = Arc<dyn Fn(&[u8]) -> io::Result<()> + Send + Sync>;

pub struct LocalNetworkCheck {
    pub send: Sender,
    pub limit: Duration,
    pub retry: Duration,
}

impl PermissionCheck for LocalNetworkCheck {
    fn code(&self) -> PermissionCode {
        PermissionCode::LocalNetwork
    }

    fn ask(&self) -> Finding {
        let query = query();
        let started = Instant::now();
        loop {
            match (self.send)(&query) {
                Ok(()) => return Finding::granted(),
                Err(error) if error.kind() == ErrorKind::HostUnreachable => {
                    if started.elapsed() >= self.limit {
                        return Finding::denied(Advice::AllowInSettings);
                    }
                    thread::sleep(self.retry);
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        ErrorKind::NetworkUnreachable | ErrorKind::AddrNotAvailable
                    ) =>
                {
                    return Finding::not_applicable(None);
                }
                Err(_) => return Finding::unknown(Advice::CheckFailed),
            }
        }
    }
}

pub fn multicast() -> Sender {
    Arc::new(|query: &[u8]| {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
        socket.send_to(query, (MDNS_GROUP, MDNS_PORT)).map(|_| ())
    })
}

pub fn query() -> Vec<u8> {
    let mut bytes = vec![0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0];
    for label in QUESTION {
        bytes.push(u8::try_from(label.len()).unwrap_or(u8::MAX));
        bytes.extend_from_slice(label.as_bytes());
    }
    bytes.push(0);
    bytes.extend_from_slice(&POINTER_TYPE.to_be_bytes());
    bytes.extend_from_slice(&INTERNET_CLASS.to_be_bytes());
    bytes
}
