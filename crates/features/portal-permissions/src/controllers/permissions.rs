use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use portal_feature::ApiError;
use portal_model::DetectedEnvironment;

use crate::responses::PermissionsResponse;
use crate::types::PermissionsState;

pub const OUTSIDE: &str = "permissions can be asked for only from inside your environments";

pub async fn show(State(state): State<PermissionsState>) -> Json<PermissionsResponse> {
    Json(PermissionsResponse::of(&state.show.run()))
}

pub async fn request(
    State(state): State<PermissionsState>,
    detected: Option<Extension<DetectedEnvironment>>,
) -> Result<StatusCode, ApiError> {
    let inside = detected.is_some_and(|Extension(detected)| !detected.environment.is_internet());
    if !inside {
        return Err(ApiError::Forbidden(OUTSIDE.to_string()));
    }
    state.request.start()?;
    Ok(StatusCode::ACCEPTED)
}
