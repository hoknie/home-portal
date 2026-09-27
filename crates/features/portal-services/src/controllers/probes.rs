use axum::Extension;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use portal_feature::ApiError;
use portal_model::Environment;

use crate::types::ServicesState;

pub async fn probe_now(
    State(state): State<ServicesState>,
    Extension(environment): Extension<Environment>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.wake.run(&id, &environment)?;
    Ok(StatusCode::ACCEPTED)
}
