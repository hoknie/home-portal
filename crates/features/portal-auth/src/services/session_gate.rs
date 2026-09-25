use std::sync::Arc;

use axum::http::HeaderMap;
use portal_config::ConfigStore;
use portal_feature::{ApiError, Gate, Principal};
use time::OffsetDateTime;

use super::SessionStore;
use crate::helpers::session_token;
use crate::types::UsersSection;

pub struct SessionGate {
    pub configuration: Arc<ConfigStore>,
    pub sessions: Arc<SessionStore>,
}

impl Gate for SessionGate {
    fn admit(&self, headers: &HeaderMap) -> Result<Principal, ApiError> {
        let token = session_token(headers).ok_or(ApiError::Unauthorized)?;
        let users = UsersSection::read(&self.configuration.read().document).unwrap_or_default();
        self.sessions
            .admit(&token, OffsetDateTime::now_utc(), |name| {
                users.find(name).is_some()
            })
            .map(|name| Principal { name })
            .ok_or(ApiError::Unauthorized)
    }
}
