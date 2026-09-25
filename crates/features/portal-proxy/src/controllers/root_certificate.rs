use axum::Extension;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use portal_feature::ApiError;
use portal_model::DetectedEnvironment;

use crate::clients::CaddyAdmin;
use crate::responses::ProxyResponse;
use crate::types::ProxyState;

pub const PEM: &str = "application/x-pem-file";
pub const ATTACHMENT: &str = "attachment; filename=\"home-portal-root.crt\"";
pub const UNKNOWN: &str = "not found";

pub async fn root_certificate(
    State(state): State<ProxyState>,
    detected: Option<Extension<DetectedEnvironment>>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let inside = detected.is_some_and(|Extension(detected)| !detected.environment.is_internet());
    if !inside && state.ports.gate.admit(&headers).is_err() {
        return Err(ApiError::NotFound(UNKNOWN));
    }
    let view = state.sync.view();
    let admin = view.settings.admin.clone();
    if !ProxyResponse::of(view).uses_internal() {
        return Err(ApiError::NotFound(UNKNOWN));
    }
    let certificate = CaddyAdmin::new(&admin)
        .map_err(ApiError::ServiceUnavailable)?
        .root_certificate()
        .await
        .map_err(|problem| ApiError::ServiceUnavailable(problem.to_string()))?;
    Ok((
        [(CONTENT_TYPE, PEM), (CONTENT_DISPOSITION, ATTACHMENT)],
        certificate,
    )
        .into_response())
}
