use axum::body::Body;
use axum::http::header::{HOST, ORIGIN};
use axum::http::{Method, Request};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use portal_feature::ApiError;

use super::json_only::API_PREFIX;

pub const FETCH_SITE: &str = "sec-fetch-site";
pub const ALLOWED_SITES: [&str; 2] = ["same-origin", "none"];
pub const CROSS_ORIGIN: &str = "a change must come from the portal's own pages";

pub async fn same_origin(request: Request<Body>, next: Next) -> Response {
    if let Some(reason) = refusal(&request) {
        return ApiError::Forbidden(reason.to_string()).into_response();
    }
    next.run(request).await
}

pub fn refusal(request: &Request<Body>) -> Option<&'static str> {
    let mutating = matches!(
        *request.method(),
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );
    if !mutating || !request.uri().path().starts_with(API_PREFIX) {
        return None;
    }
    let headers = request.headers();
    let text = |name: &str| headers.get(name).and_then(|value| value.to_str().ok());
    if let Some(site) = text(FETCH_SITE) {
        let own = ALLOWED_SITES
            .iter()
            .any(|allowed| site.eq_ignore_ascii_case(allowed));
        return (!own).then_some(CROSS_ORIGIN);
    }
    let host = text(HOST.as_str());
    let foreign_origin = text(ORIGIN.as_str()).is_some_and(|origin| {
        let authority = origin
            .split_once("://")
            .map_or(origin, |(_, rest)| rest)
            .trim_end_matches('/');
        host.is_none_or(|host| !authority.eq_ignore_ascii_case(host))
    });
    foreign_origin.then_some(CROSS_ORIGIN)
}
