use std::collections::BTreeSet;
use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use portal_config::ConfigStore;
use portal_feature::ApiError;

use crate::responses::{SecretResponse, SecretsResponse};
use crate::services::referenced_secrets;

pub async fn list(
    State(configuration): State<Arc<ConfigStore>>,
) -> Result<Json<SecretsResponse>, ApiError> {
    let snapshot = configuration.read();
    let defined: BTreeSet<String> = configuration.secret_names().into_iter().collect();
    let mut names = referenced_secrets(&snapshot.document);
    names.extend(defined.iter().cloned());
    let secrets = names
        .into_iter()
        .map(|name| SecretResponse {
            set: defined.contains(&name),
            name,
        })
        .collect();
    Ok(Json(SecretsResponse { secrets }))
}
