use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::routing::{delete, post};
use portal_config::{ConfigStore, Storage};
use portal_feature::{EventSink, Feature, Gate, Loop, Validator};
use time::OffsetDateTime;

use crate::controllers::{sign_in, sign_out, who_am_i};
use crate::ports::Connection;
use crate::repositories::SessionFile;
use crate::services::{SessionGate, SessionStore, Throttle, validate_users};
use crate::types::AuthState;

pub struct AuthFeature {
    state: AuthState,
}

impl AuthFeature {
    pub const NAME: &'static str = "auth";
    pub const PATH: &'static str = "/api/session";
    pub const PRUNE_EVERY: Duration = Duration::from_secs(300);

    pub fn new(
        configuration: Arc<ConfigStore>,
        connection: Arc<dyn Connection>,
        events: Arc<dyn EventSink>,
    ) -> AuthFeature {
        let sessions = SessionStore::kept_in(
            SessionFile::at(configuration.storage(Storage::Sessions)),
            OffsetDateTime::now_utc(),
        );
        AuthFeature {
            state: AuthState {
                configuration,
                sessions: Arc::new(sessions),
                throttle: Arc::new(Throttle::default()),
                connection,
                events,
            },
        }
    }

    pub fn gate(&self) -> Arc<dyn Gate> {
        Arc::new(SessionGate {
            configuration: self.state.configuration.clone(),
            sessions: self.state.sessions.clone(),
        })
    }
}

impl Feature for AuthFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::PATH, delete(sign_out))
            .with_state(self.state.clone())
    }

    fn public_router(&self) -> Router {
        Router::new()
            .route(Self::PATH, post(sign_in).get(who_am_i))
            .with_state(self.state.clone())
    }

    fn validator(&self) -> Option<Validator> {
        Some(validate_users)
    }

    fn loops(&self) -> Vec<Loop> {
        let sessions = self.state.sessions.clone();
        vec![Box::pin(async move {
            let mut ticks = tokio::time::interval(Self::PRUNE_EVERY);
            loop {
                ticks.tick().await;
                sessions.prune(OffsetDateTime::now_utc());
            }
        })]
    }

    fn stop(&self) {
        self.state.sessions.flush();
    }
}
