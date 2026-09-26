use std::sync::Arc;

use axum::Extension;
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use portal_feature::ClientAddress;
use portal_model::Environment;

use crate::requests::DnsQueryRequest;
use crate::services::{Library, respond};

pub const DNS_MESSAGE: &str = "application/dns-message";
pub const NOT_A_MESSAGE: &str = "the body is not a DNS message";
pub const NOT_SERVED: &str = "DNS over HTTPS is off";
pub const MISSING_PARAMETER: &str = "the dns parameter must hold a base64url DNS message";

pub async fn resolve_posted(
    State(library): State<Arc<Library>>,
    client: Option<Extension<ClientAddress>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let typed = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case(DNS_MESSAGE));
    if !typed {
        return (StatusCode::BAD_REQUEST, NOT_A_MESSAGE).into_response();
    }
    resolve(&library, client, &body)
}

pub async fn resolve_encoded(
    State(library): State<Arc<Library>>,
    client: Option<Extension<ClientAddress>>,
    Query(request): Query<DnsQueryRequest>,
) -> Response {
    let Some(bytes) = request
        .dns
        .and_then(|text| URL_SAFE_NO_PAD.decode(text.trim_end_matches('=')).ok())
    else {
        return (StatusCode::BAD_REQUEST, MISSING_PARAMETER).into_response();
    };
    resolve(&library, client, &bytes)
}

fn resolve(library: &Library, client: Option<Extension<ClientAddress>>, bytes: &[u8]) -> Response {
    let settings = library.settings();
    if !(settings.enabled && settings.https.enabled) {
        return (StatusCode::NOT_FOUND, NOT_SERVED).into_response();
    }
    let book = library.book();
    let environment = client
        .map(|Extension(ClientAddress(address))| book.environment_of(address))
        .unwrap_or_else(Environment::internet);
    match respond(&book, &environment, bytes, None) {
        None => (StatusCode::BAD_REQUEST, NOT_A_MESSAGE).into_response(),
        Some(answer) => (
            StatusCode::OK,
            [
                (CONTENT_TYPE, DNS_MESSAGE.to_string()),
                (CACHE_CONTROL, format!("max-age={}", book.ttl)),
            ],
            answer,
        )
            .into_response(),
    }
}
