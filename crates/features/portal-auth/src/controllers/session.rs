use std::net::SocketAddr;

use axum::extract::{ConnectInfo, State};
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use portal_feature::{ApiError, Gate};
use time::OffsetDateTime;

use crate::helpers::{clear_cookie, session_cookie, session_token, verify_password};
use crate::requests::SignInRequest;
use crate::responses::SessionResponse;
use crate::services::SessionGate;
use crate::types::{AuthState, UsersSection};

pub async fn sign_in(
    State(state): State<AuthState>,
    connect: Option<Extension<ConnectInfo<SocketAddr>>>,
    headers: HeaderMap,
    Json(request): Json<SignInRequest>,
) -> Result<Response, ApiError> {
    let now = OffsetDateTime::now_utc();
    let peer = connect.map(|Extension(ConnectInfo(address))| address);
    let client = state.connection.client_address(peer, &headers);
    state
        .throttle
        .check(client, now)
        .map_err(|retry_after_seconds| ApiError::TooManyRequests {
            retry_after_seconds,
        })?;
    let users = UsersSection::read(&state.configuration.read().document)
        .map_err(|message| ApiError::Internal(format!("users section: {message}")))?;
    let hash = users
        .find(&request.name)
        .map(|user| user.password_hash.clone());
    let password = request.password;
    let verified = tokio::task::spawn_blocking(move || verify_password(&password, hash.as_deref()))
        .await
        .map_err(|error| ApiError::Internal(format!("password check: {error}")))?;
    if !verified {
        state.throttle.fail(client, now);
        return Err(ApiError::Unauthorized);
    }
    state.throttle.succeed(client);
    let token = state.sessions.create(&request.name, now);
    let mut response = Json(SessionResponse { name: request.name }).into_response();
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
    let gate = SessionGate {
        configuration: state.configuration.clone(),
        sessions: state.sessions.clone(),
    };
    let principal = gate.admit(&headers)?;
    Ok(Json(SessionResponse {
        name: principal.name,
    }))
}

pub async fn sign_out(
    State(state): State<AuthState>,
    connect: Option<Extension<ConnectInfo<SocketAddr>>>,
    headers: HeaderMap,
) -> Response {
    let peer = connect.map(|Extension(ConnectInfo(address))| address);
    if let Some(token) = session_token(&headers) {
        state.sessions.end(&token);
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        clear_cookie(&state.connection.cookie_scope(peer, &headers)),
    );
    response
}
