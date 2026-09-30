use std::net::IpAddr;
use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::{ApiError, EventName, EventSink, PortalEvent, Visitor};
use time::OffsetDateTime;

use crate::services::{PasswordChecks, SessionStore};
use crate::types::UsersSection;

#[derive(Clone)]
pub struct SignIn {
    configuration: Arc<ConfigStore>,
    sessions: Arc<SessionStore>,
    checks: Arc<PasswordChecks>,
    events: Arc<dyn EventSink>,
}

impl SignIn {
    pub fn new(
        configuration: Arc<ConfigStore>,
        sessions: Arc<SessionStore>,
        (events, checks): (Arc<dyn EventSink>, Arc<PasswordChecks>),
    ) -> SignIn {
        SignIn {
            configuration,
            sessions,
            checks,
            events,
        }
    }

    pub async fn run(
        &self,
        visitor: Visitor,
        client: IpAddr,
        password: String,
    ) -> Result<String, ApiError> {
        let now = OffsetDateTime::now_utc();
        let users = UsersSection::read(&self.configuration.read().document)
            .map_err(|message| ApiError::Internal(format!("users section: {message}")))?;
        let hash = users
            .find(&visitor.user)
            .map(|user| user.password_hash.clone());
        let credential = users.credential(&visitor.user).unwrap_or_default();
        let verified = match self.checks.verify(client, password, hash).await {
            Ok(verified) => verified,
            Err(ApiError::TooManyRequests {
                retry_after_seconds,
            }) => {
                self.announce(
                    EventName::UserSignInFailed,
                    &Visitor {
                        reason: Some(PortalEvent::THROTTLED),
                        ..visitor
                    },
                );
                return Err(ApiError::TooManyRequests {
                    retry_after_seconds,
                });
            }
            Err(other) => return Err(other),
        };
        if !verified {
            self.announce(
                EventName::UserSignInFailed,
                &Visitor {
                    reason: Some(PortalEvent::CREDENTIALS),
                    ..visitor
                },
            );
            return Err(ApiError::Unauthorized);
        }
        let token = self.sessions.create(&visitor.user, &credential, now);
        self.announce(EventName::UserSignedIn, &visitor);
        Ok(token)
    }

    fn announce(&self, name: EventName, visitor: &Visitor) {
        self.events.emit(PortalEvent::visited(
            name,
            visitor,
            OffsetDateTime::now_utc(),
        ));
    }
}
