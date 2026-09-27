use portal_feature::{ApiError, Principal};

use crate::services::SessionGate;

#[derive(Clone)]
pub struct ShowSession {
    gate: SessionGate,
}

impl ShowSession {
    pub fn new(gate: SessionGate) -> ShowSession {
        ShowSession { gate }
    }

    pub fn run(&self, token: Option<&str>) -> Result<Principal, ApiError> {
        token
            .and_then(|token| self.gate.principal_of(token))
            .ok_or(ApiError::Unauthorized)
    }
}
