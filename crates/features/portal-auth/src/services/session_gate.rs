use std::sync::Arc;

use axum::http::HeaderMap;
use portal_config::ConfigStore;
use portal_feature::{ApiError, Gate, Principal};
use time::OffsetDateTime;

use super::{SessionStore, principal_for};
use crate::helpers::session_token;
use crate::types::UsersSection;

#[derive(Clone)]
pub struct SessionGate {
    pub configuration: Arc<ConfigStore>,
    pub sessions: Arc<SessionStore>,
}

impl SessionGate {
    pub fn principal_of(&self, token: &str) -> Option<Principal> {
        let users = self
            .configuration
            .read()
            .typed(UsersSection::read)
            .unwrap_or_default();
        self.sessions
            .admit(token, OffsetDateTime::now_utc(), |name| {
                users.credential(name)
            })
            .map(|name| principal_for(&users, &name))
    }
}

impl Gate for SessionGate {
    fn admit(&self, headers: &HeaderMap) -> Result<Principal, ApiError> {
        let token = session_token(headers).ok_or(ApiError::Unauthorized)?;
        self.principal_of(&token).ok_or(ApiError::Unauthorized)
    }
}
