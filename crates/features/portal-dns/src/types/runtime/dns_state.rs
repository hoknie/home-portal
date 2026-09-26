use super::TransportState;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DnsState {
    pub plain: TransportState,
    pub tls: TransportState,
    pub last_error: Option<String>,
}
