use std::time::Duration;

use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use portal_feature::ApiError;

use super::json_only::API_PREFIX;

pub const API_DEADLINE: Duration = Duration::from_secs(60);

pub async fn deadline(request: Request<Body>, next: Next) -> Response {
    if !request.uri().path().starts_with(API_PREFIX) {
        return next.run(request).await;
    }
    match tokio::time::timeout(API_DEADLINE, next.run(request)).await {
        Ok(response) => response,
        Err(_) => ApiError::ServiceUnavailable(format!(
            "the request was not answered within {} seconds",
            API_DEADLINE.as_secs()
        ))
        .into_response(),
    }
}
