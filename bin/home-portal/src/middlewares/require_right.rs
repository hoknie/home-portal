use std::sync::Arc;

use axum::body::Body;
use axum::extract::{MatchedPath, State};
use axum::http::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use portal_feature::{ApiError, Principal};

use crate::types::RuleBook;

pub const NO_RULE: &str = "this route declares no right";

pub async fn require_right(
    State(book): State<Arc<RuleBook>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    match refusal(&book, &request) {
        Some(error) => error.into_response(),
        None => next.run(request).await,
    }
}

pub fn refusal(book: &RuleBook, request: &Request<Body>) -> Option<ApiError> {
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| request.uri().path(), MatchedPath::as_str);
    let Some(requirement) = book.requirement(request.method(), path) else {
        tracing::error!(method = %request.method(), path, "a protected route has no rule");
        return Some(ApiError::Forbidden(NO_RULE.to_string()));
    };
    let allowed = request
        .extensions()
        .get::<Principal>()
        .is_some_and(|principal| requirement.met_by(&principal.rights));
    (!allowed).then(|| ApiError::Forbidden(format!("needs {}", requirement.describe())))
}
