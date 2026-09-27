use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, State};
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use portal_feature::{ApiError, Visitor};
use portal_model::{DetectedEnvironment, Environment};

use crate::helpers::{clear_cookie, session_cookie, session_token};
use crate::requests::SignInRequest;
use crate::responses::SessionResponse;
use crate::types::{AuthState, CookieScope};

pub async fn sign_in(
    State(state): State<AuthState>,
    connect: Option<Extension<ConnectInfo<SocketAddr>>>,
    detected: Option<Extension<DetectedEnvironment>>,
    headers: HeaderMap,
    Json(request): Json<SignInRequest>,
) -> Result<Response, ApiError> {
    let peer = connect.map(|Extension(ConnectInfo(address))| address);
    let client = state.connection.client_address(peer, &headers);
    let visitor = visitor_of(request.name.clone(), client, detected.as_ref());
    let token = state
        .sign_in
        .run(visitor, client, request.password.clone())
        .await?;
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
    connect: Option<Extension<ConnectInfo<SocketAddr>>>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = session_token(&headers);
    let principal = state.session.run(token.as_deref())?;
    let peer = connect.map(|Extension(ConnectInfo(address))| address);
    let mut response = Json(SessionResponse {
        name: principal.name,
    })
    .into_response();
    let scope = state.connection.cookie_scope(peer, &headers);
    if scope.domain.is_some()
        && let Some(token) = token
    {
        response
            .headers_mut()
            .insert(SET_COOKIE, session_cookie(&token, &scope));
    }
    Ok(response)
}

pub async fn sign_out(
    State(state): State<AuthState>,
    connect: Option<Extension<ConnectInfo<SocketAddr>>>,
    detected: Option<Extension<DetectedEnvironment>>,
    headers: HeaderMap,
) -> Response {
    let peer = connect.map(|Extension(ConnectInfo(address))| address);
    let client = state.connection.client_address(peer, &headers);
    let token = session_token(&headers);
    state.sign_out.run(
        token.as_deref(),
        visitor_of(String::new(), client, detected.as_ref()),
    );
    let mut response = StatusCode::NO_CONTENT.into_response();
    let scope = state.connection.cookie_scope(peer, &headers);
    response
        .headers_mut()
        .append(SET_COOKIE, clear_cookie(&scope));
    if scope.domain.is_some() {
        response.headers_mut().append(
            SET_COOKIE,
            clear_cookie(&CookieScope {
                domain: None,
                ..scope
            }),
        );
    }
    response
}

fn visitor_of(
    user: String,
    client: IpAddr,
    detected: Option<&Extension<DetectedEnvironment>>,
) -> Visitor {
    Visitor {
        user,
        address: client.to_string(),
        environment: detected
            .map(|Extension(detected)| detected.environment.as_str().to_string())
            .unwrap_or_else(|| Environment::INTERNET.to_string()),
        reason: None,
    }
}
