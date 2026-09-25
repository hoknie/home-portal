use axum::Extension;
use axum::Json;
use axum::extract::State;
use portal_feature::ApiError;
use portal_model::{DetectedEnvironment, Environment};

use crate::responses::EnvironmentResponse;
use crate::services::read_environments;
use crate::types::NetworkState;

pub async fn show_environment(
    State(state): State<NetworkState>,
    Extension(environment): Extension<Environment>,
    Extension(detected): Extension<DetectedEnvironment>,
) -> Result<Json<EnvironmentResponse>, ApiError> {
    let environments = read_environments(&state.configuration.read().document)
        .map_err(|errors| ApiError::Internal(format!("environments: {}", errors.len())))?;
    Ok(Json(EnvironmentResponse {
        environment,
        switchable: detected.switchable(),
        detected: detected.environment,
        environments: environments.names(),
    }))
}
