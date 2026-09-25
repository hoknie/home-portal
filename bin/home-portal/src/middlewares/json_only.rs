use axum::body::Body;
use axum::http::header::{CONTENT_LENGTH, CONTENT_TYPE, TRANSFER_ENCODING};
use axum::http::{Method, Request};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use portal_feature::ApiError;

pub const API_PREFIX: &str = "/api/";
pub const JSON: &str = "application/json";

pub async fn json_only(request: Request<Body>, next: Next) -> Response {
    let mutating = matches!(
        *request.method(),
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );
    let headers = request.headers();
    let has_body = headers.contains_key(TRANSFER_ENCODING)
        || headers
            .get(CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|length| length != "0");
    let json = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.trim_start().starts_with(JSON));
    if mutating && has_body && !json && request.uri().path().starts_with(API_PREFIX) {
        return ApiError::UnsupportedMediaType.into_response();
    }
    next.run(request).await
}
