use std::net::SocketAddr;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransportState {
    pub listening: bool,
    pub address: Option<SocketAddr>,
    pub reason: Option<String>,
}

impl TransportState {
    pub fn off(reason: &str) -> TransportState {
        TransportState {
            listening: false,
            address: None,
            reason: Some(reason.to_string()),
        }
    }
}
