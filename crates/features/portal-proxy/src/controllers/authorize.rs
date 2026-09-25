use std::net::SocketAddr;

use axum::Extension;
use axum::extract::{ConnectInfo, State};
use axum::http::header::LOCATION;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use portal_feature::ApiError;
use portal_model::DetectedEnvironment;

use crate::helpers::{FORWARDED_METHOD, FORWARDED_URI, forwarded, forwarded_host, sign_in_address};
use crate::renderers::USER_HEADER;
use crate::types::ProxyState;

pub const UNKNOWN: &str = "not found";

pub async fn authorize(
    State(state): State<ProxyState>,
    peer: Option<Extension<ConnectInfo<SocketAddr>>>,
    detected: Option<Extension<DetectedEnvironment>>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let trusted =
        peer.is_some_and(|Extension(ConnectInfo(peer))| state.ports.peers.trusts(peer.ip()));
    let settings = state.sync.settings();
    let portal_host = settings
        .active()
        .filter(|_| trusted)
        .ok_or(ApiError::NotFound(UNKNOWN))?;
    let host = forwarded_host(&headers).ok_or(ApiError::NotFound(UNKNOWN))?;
    let service = state
        .ports
        .services
        .published()
        .into_iter()
        .find(|service| service.publication.host == host)
        .ok_or(ApiError::NotFound(UNKNOWN))?;
    let environment = detected.map(|Extension(detected)| detected.environment);
    let needs_sign_in = environment.is_none_or(|environment| {
        service
            .publication
            .auth
            .iter()
            .any(|name| name == environment.as_str())
    });
    match state.ports.gate.admit(&headers) {
        Ok(principal) => Ok(passed(Some(&principal.name))),
        Err(_) if !needs_sign_in => Ok(passed(None)),
        Err(_) if reads(&headers) => {
            let uri = forwarded(&headers, FORWARDED_URI);
            let location = sign_in_address(
                &settings.origin(portal_host),
                &settings.origin(&host),
                uri.as_deref(),
            );
            let location = HeaderValue::from_str(&location)
                .map_err(|error| ApiError::Internal(format!("sign-in address: {error}")))?;
            Ok((StatusCode::FOUND, [(LOCATION, location)]).into_response())
        }
        Err(_) => Err(ApiError::Unauthorized),
    }
}

fn reads(headers: &HeaderMap) -> bool {
    forwarded(headers, FORWARDED_METHOD).is_none_or(|method| {
        method.eq_ignore_ascii_case(Method::GET.as_str())
            || method.eq_ignore_ascii_case(Method::HEAD.as_str())
    })
}

fn passed(user: Option<&str>) -> Response {
    let mut response = StatusCode::OK.into_response();
    if let Some(value) = user.and_then(|user| HeaderValue::from_str(user).ok()) {
        response.headers_mut().insert(USER_HEADER, value);
    }
    response
}
