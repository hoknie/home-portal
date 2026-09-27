use axum::Json;
use axum::extract::State;
use portal_feature::ApiError;

use crate::responses::{SecretResponse, SecretsResponse};
use crate::types::SecretsState;

pub async fn list(State(state): State<SecretsState>) -> Result<Json<SecretsResponse>, ApiError> {
    let secrets = state
        .list
        .run()
        .into_iter()
        .map(|entry| SecretResponse {
            name: entry.name,
            set: entry.set,
        })
        .collect();
    Ok(Json(SecretsResponse { secrets }))
}
