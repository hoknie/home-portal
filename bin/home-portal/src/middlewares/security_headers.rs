use axum::body::Body;
use axum::http::header::{
    CONTENT_SECURITY_POLICY, REFERRER_POLICY, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS,
};
use axum::http::{HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;

use super::json_only::API_PREFIX;

pub const INTERFACE_POLICY: &str = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: https:; font-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'";
pub const API_POLICY: &str = "default-src 'none'; frame-ancestors 'none'";
pub const TRANSPORT: &str = "max-age=31536000";
pub const FORWARDED_PROTO: &str = "x-forwarded-proto";

pub async fn security_headers(request: Request<Body>, next: Next) -> Response {
    let api = request.uri().path().starts_with(API_PREFIX);
    let secure = request.uri().scheme_str() == Some("https")
        || request
            .headers()
            .get(FORWARDED_PROTO)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|proto| proto.eq_ignore_ascii_case("https"));
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(REFERRER_POLICY, HeaderValue::from_static("same-origin"));
    if !headers.contains_key(CONTENT_SECURITY_POLICY) {
        let policy = if api { API_POLICY } else { INTERFACE_POLICY };
        headers.insert(CONTENT_SECURITY_POLICY, HeaderValue::from_static(policy));
    }
    if secure {
        headers.insert(
            STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static(TRANSPORT),
        );
    }
    response
}
