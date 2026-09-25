use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, State};
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use portal_feature::{ApiError, EventName, Gate, PortalEvent, Visitor};
use portal_model::{DetectedEnvironment, Environment};
use time::OffsetDateTime;

use crate::helpers::{clear_cookie, session_cookie, session_token, verify_password};
use crate::requests::SignInRequest;
use crate::responses::SessionResponse;
use crate::services::SessionGate;
use crate::types::{AuthState, UsersSection};

pub async fn sign_in(
    State(state): State<AuthState>,
    connect: Option<Extension<ConnectInfo<SocketAddr>>>,
    detected: Option<Extension<DetectedEnvironment>>,
    headers: HeaderMap,
    Json(request): Json<SignInRequest>,
) -> Result<Response, ApiError> {
    let now = OffsetDateTime::now_utc();
    let peer = connect.map(|Extension(ConnectInfo(address))| address);
    let client = state.connection.client_address(peer, &headers);
    let visitor = |reason: Option<&'static str>| Visitor {
        user: request.name.clone(),
        address: client.to_string(),
        environment: environment_of(detected.as_ref()),
        reason,
    };
    if let Err(retry_after_seconds) = state.throttle.check(client, now) {
        announce(
            &state,
            EventName::UserSignInFailed,
            &visitor(Some(PortalEvent::THROTTLED)),
        );
        return Err(ApiError::TooManyRequests {
            retry_after_seconds,
        });
    }
    let users = UsersSection::read(&state.configuration.read().document)
        .map_err(|message| ApiError::Internal(format!("users section: {message}")))?;
    let hash = users
        .find(&request.name)
        .map(|user| user.password_hash.clone());
    let password = request.password.clone();
    let verified = tokio::task::spawn_blocking(move || verify_password(&password, hash.as_deref()))
        .await
        .map_err(|error| ApiError::Internal(format!("password check: {error}")))?;
    if !verified {
        state.throttle.fail(client, now);
        announce(
            &state,
            EventName::UserSignInFailed,
            &visitor(Some(PortalEvent::CREDENTIALS)),
        );
        return Err(ApiError::Unauthorized);
    }
    state.throttle.succeed(client);
    let token = state.sessions.create(&request.name, now);
    announce(&state, EventName::UserSignedIn, &visitor(None));
    let mut response = Json(SessionResponse {
        name: request.name.clone(),
    })
    .into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        session_cookie(&token, &state.connection.cookie_scope(peer, &headers)),
    );
    Ok(response)
}

pub async fn who_am_i(
    State(state): State<AuthState>,
    headers: HeaderMap,
) -> Result<Json<SessionResponse>, ApiError> {
    let principal = gate_of(&state).admit(&headers)?;
    Ok(Json(SessionResponse {
        name: principal.name,
    }))
}

pub async fn sign_out(
    State(state): State<AuthState>,
    connect: Option<Extension<ConnectInfo<SocketAddr>>>,
    detected: Option<Extension<DetectedEnvironment>>,
    headers: HeaderMap,
) -> Response {
    let peer = connect.map(|Extension(ConnectInfo(address))| address);
    let signed_in = gate_of(&state).admit(&headers).ok();
    if let Some(token) = session_token(&headers) {
        state.sessions.end(&token);
    }
    if let Some(principal) = signed_in {
        let client: IpAddr = state.connection.client_address(peer, &headers);
        announce(
            &state,
            EventName::UserSignedOut,
            &Visitor {
                user: principal.name,
                address: client.to_string(),
                environment: environment_of(detected.as_ref()),
                reason: None,
            },
        );
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        clear_cookie(&state.connection.cookie_scope(peer, &headers)),
    );
    response
}

fn gate_of(state: &AuthState) -> SessionGate {
    SessionGate {
        configuration: state.configuration.clone(),
        sessions: state.sessions.clone(),
    }
}

fn environment_of(detected: Option<&Extension<DetectedEnvironment>>) -> String {
    detected
        .map(|Extension(detected)| detected.environment.as_str().to_string())
        .unwrap_or_else(|| Environment::INTERNET.to_string())
}

fn announce(state: &AuthState, name: EventName, visitor: &Visitor) {
    state.events.emit(PortalEvent::visited(
        name,
        visitor,
        OffsetDateTime::now_utc(),
    ));
}
