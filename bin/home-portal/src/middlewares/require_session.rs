use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use portal_feature::Gate;

pub async fn require_session(
    State(gate): State<Arc<dyn Gate>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    match gate.admit(request.headers()) {
        Ok(principal) => {
            request.extensions_mut().insert(principal);
            next.run(request).await
        }
        Err(error) => error.into_response(),
    }
}
