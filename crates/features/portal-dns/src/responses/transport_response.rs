use serde::Serialize;

use crate::types::TransportState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TransportResponse {
    pub listening: bool,
    pub address: Option<String>,
    pub reason: Option<String>,
}

impl TransportResponse {
    pub fn of(state: &TransportState) -> TransportResponse {
        TransportResponse {
            listening: state.listening,
            address: state.address.map(|address| address.to_string()),
            reason: state.reason.clone(),
        }
    }
}
