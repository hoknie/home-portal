use std::net::IpAddr;
use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::{ApiError, EventName, EventSink, PortalEvent, Visitor};
use time::OffsetDateTime;

use crate::helpers::verify_password;
use crate::services::{SessionStore, Throttle};
use crate::types::UsersSection;

#[derive(Clone)]
pub struct SignIn {
    configuration: Arc<ConfigStore>,
    sessions: Arc<SessionStore>,
    throttle: Arc<Throttle>,
    events: Arc<dyn EventSink>,
}

impl SignIn {
    pub fn new(
        configuration: Arc<ConfigStore>,
        sessions: Arc<SessionStore>,
        events: Arc<dyn EventSink>,
    ) -> SignIn {
        SignIn {
            configuration,
            sessions,
            throttle: Arc::new(Throttle::default()),
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
        if let Err(retry_after_seconds) = self.throttle.check(client, now) {
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
        let users = UsersSection::read(&self.configuration.read().document)
            .map_err(|message| ApiError::Internal(format!("users section: {message}")))?;
        let hash = users
            .find(&visitor.user)
            .map(|user| user.password_hash.clone());
        let verified =
            tokio::task::spawn_blocking(move || verify_password(&password, hash.as_deref()))
                .await
                .map_err(|error| ApiError::Internal(format!("password check: {error}")))?;
        if !verified {
            self.throttle.fail(client, now);
            self.announce(
                EventName::UserSignInFailed,
                &Visitor {
                    reason: Some(PortalEvent::CREDENTIALS),
                    ..visitor
                },
            );
            return Err(ApiError::Unauthorized);
        }
        self.throttle.succeed(client);
        let token = self.sessions.create(&visitor.user, now);
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
