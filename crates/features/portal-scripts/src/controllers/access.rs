use axum::Extension;
use axum::http::header::ETAG;
use axum::response::{IntoResponse, Response};
use portal_config::Revision;
use portal_feature::{ApiError, Principal};
use portal_model::DetectedEnvironment;

use crate::types::ScriptsState;

pub const OFF: &str = "not found";
pub const OUTSIDE: &str = "scripts can be changed only from inside your environments";

pub fn switched_on(state: &ScriptsState) -> Result<(), ApiError> {
    if state.editing.run() {
        Ok(())
    } else {
        Err(ApiError::NotFound(OFF))
    }
}

pub fn inside(detected: Option<&Extension<DetectedEnvironment>>) -> bool {
    detected.is_some_and(|Extension(detected)| !detected.environment.is_internet())
}

pub fn writable(
    state: &ScriptsState,
    detected: Option<&Extension<DetectedEnvironment>>,
) -> Result<(), ApiError> {
    switched_on(state)?;
    if inside(detected) {
        Ok(())
    } else {
        Err(ApiError::Forbidden(OUTSIDE.to_string()))
    }
}

pub fn name_of(principal: Option<Extension<Principal>>) -> String {
    principal.map_or_else(
        || "someone".to_string(),
        |Extension(principal)| principal.name,
    )
}

pub fn logged(action: &str, user: &str, path: &str) {
    tracing::info!(action, user, path, "script changed");
}

pub fn tagged(revision: &Revision, body: impl IntoResponse) -> Response {
    let mut response = body.into_response();
    response.headers_mut().insert(ETAG, revision.etag());
    response
}

pub async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, ApiError> + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|error| ApiError::Internal(error.to_string()))?
}
