use axum::Json;
use axum::extract::State;
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use portal_config::Revision;
use portal_feature::ApiError;

use crate::repositories::{NETWORK, SECTION, trust_loopback, write_choice};
use crate::requests::ProxySettingsRequest;
use crate::responses::ProxyResponse;
use crate::types::ProxyState;

pub const DISABLED: &str = "the proxy is not enabled in the configuration";

pub async fn show(State(state): State<ProxyState>) -> Response {
    answer(&state, StatusCode::OK)
}

pub async fn apply(State(state): State<ProxyState>) -> Result<Response, ApiError> {
    if !state.sync.settings().enabled {
        return Err(ApiError::Conflict(DISABLED.to_string()));
    }
    state
        .sync
        .apply()
        .await
        .map_err(|problem| ApiError::BadGateway(problem.to_string()))?;
    Ok(answer(&state, StatusCode::OK))
}

pub async fn change(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(request): Json<ProxySettingsRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let choice = request.into_choice();
    let snapshot = state.configuration.read();
    let target = snapshot
        .origins
        .table(SECTION)
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| state.configuration.writes_to());
    let network_here = snapshot
        .origins
        .table(NETWORK)
        .is_none_or(|origin| origin == target);
    state
        .configuration
        .update(&target, &revision, |document| {
            write_choice(document, &choice);
            if choice.enabled && network_here {
                trust_loopback(document);
            }
            Ok(())
        })
        .await?;
    Ok(answer(&state, StatusCode::OK))
}

pub fn answer(state: &ProxyState, status: StatusCode) -> Response {
    let revision = state.configuration.read().revision;
    let mut response = (status, Json(ProxyResponse::of(state.sync.view()))).into_response();
    response.headers_mut().insert(ETAG, revision.etag());
    response
}
