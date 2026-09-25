use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::header::ETAG;
use axum::response::{IntoResponse, Response};
use portal_config::{Revision, Snapshot};
use portal_feature::ApiError;

use crate::helpers::host_interfaces;
use crate::repositories::{SECTION, write_network};
use crate::requests::NetworkRequest;
use crate::responses::NetworkResponse;
use crate::services::{check_network, read_network};
use crate::types::NetworkState;

pub async fn show(State(state): State<NetworkState>) -> Result<Response, ApiError> {
    let snapshot = state.configuration.read();
    answer(&state, &snapshot)
}

pub async fn change(
    State(state): State<NetworkState>,
    headers: HeaderMap,
    Json(request): Json<NetworkRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let settings = check_network(&request.into_raw()).map_err(ApiError::Invalid)?;
    let snapshot = state.configuration.read();
    let target = snapshot
        .origins
        .table(SECTION)
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| state.configuration.writes_to());
    let (_, snapshot) = state
        .configuration
        .update(&target, &revision, |document| {
            write_network(document, &settings);
            Ok(())
        })
        .await?;
    answer(&state, &snapshot)
}

fn answer(state: &NetworkState, snapshot: &Snapshot) -> Result<Response, ApiError> {
    let configured = read_network(&snapshot.document).map_err(|errors| {
        ApiError::Internal(format!(
            "network section: {}",
            errors
                .first()
                .map(|error| error.message.as_str())
                .unwrap_or_default()
        ))
    })?;
    let body = NetworkResponse::of(configured, state.effective, host_interfaces());
    let mut response = Json(body).into_response();
    response
        .headers_mut()
        .insert(ETAG, snapshot.revision.etag());
    Ok(response)
}
