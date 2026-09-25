use axum::Extension;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use portal_feature::ApiError;
use portal_model::Environment;

use crate::types::{ServicesSection, ServicesState, Wake};

pub const UNKNOWN_SERVICE: &str = "no such service";
pub const PROBING_DISABLED: &str = "probing is disabled for this service";

pub async fn probe_now(
    State(state): State<ServicesState>,
    Extension(environment): Extension<Environment>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let document = state.configuration.read().document;
    if ServicesSection::visible(&document, &id, &environment).is_none() {
        return Err(ApiError::NotFound(UNKNOWN_SERVICE));
    }
    match state.supervisor.wake(&id) {
        Wake::Woken => Ok(StatusCode::ACCEPTED),
        Wake::NotFound => Err(ApiError::NotFound(UNKNOWN_SERVICE)),
        Wake::Disabled => Err(ApiError::Conflict(PROBING_DISABLED.to_string())),
        Wake::TooSoon(wait) => Err(ApiError::TooManyRequests {
            retry_after_seconds: wait.as_secs().max(1),
        }),
    }
}
