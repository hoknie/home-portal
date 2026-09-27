use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::routing::{delete, get, post, put};
use portal_config::{ConfigStore, Storage};
use portal_feature::{EventSink, Feature, Gate, Loop, Validator};
use time::OffsetDateTime;

use crate::controllers::{
    change_password, create_user, delete_user, list_users, sign_in, sign_out, who_am_i,
};
use crate::ports::Connection;
use crate::repositories::SessionFile;
use crate::services::{SessionGate, SessionStore, validate_users};
use crate::types::AuthState;
use crate::usecases::{
    ChangePassword, CreateUser, DeleteUser, ListUsers, ShowSession, SignIn, SignOut,
};

pub struct AuthFeature {
    state: AuthState,
    gate: SessionGate,
}

impl AuthFeature {
    pub const NAME: &'static str = "auth";
    pub const PATH: &'static str = "/api/session";
    pub const USERS: &'static str = "/api/users";
    pub const USER: &'static str = "/api/users/{name}";
    pub const PASSWORD: &'static str = "/api/users/{name}/password";
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
        let sessions = Arc::new(sessions);
        let gate = SessionGate {
            configuration: configuration.clone(),
            sessions: sessions.clone(),
        };
        AuthFeature {
            state: AuthState {
                sign_in: SignIn::new(configuration.clone(), sessions.clone(), events.clone()),
                session: ShowSession::new(gate.clone()),
                sign_out: SignOut::new(gate.clone(), events),
                list_users: ListUsers::new(configuration.clone()),
                create_user: CreateUser::new(configuration.clone()),
                change_password: ChangePassword::new(configuration.clone(), sessions),
                delete_user: DeleteUser::new(configuration),
                connection,
            },
            gate,
        }
    }

    pub fn gate(&self) -> Arc<dyn Gate> {
        Arc::new(self.gate.clone())
    }
}

impl Feature for AuthFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::PATH, delete(sign_out))
            .route(Self::USERS, get(list_users).post(create_user))
            .route(Self::USER, delete(delete_user))
            .route(Self::PASSWORD, put(change_password))
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
        let sessions = self.gate.sessions.clone();
        vec![Box::pin(async move {
            let mut ticks = tokio::time::interval(Self::PRUNE_EVERY);
            loop {
                ticks.tick().await;
                sessions.prune(OffsetDateTime::now_utc());
            }
        })]
    }

    fn stop(&self) {
        self.gate.sessions.flush();
    }
}
