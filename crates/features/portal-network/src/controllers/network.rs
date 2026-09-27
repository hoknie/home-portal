use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::header::ETAG;
use axum::response::{IntoResponse, Response};
use portal_config::{Revision, Revisioned};
use portal_feature::ApiError;

use crate::helpers::host_interfaces;
use crate::requests::NetworkRequest;
use crate::responses::NetworkResponse;
use crate::types::{NetworkSettings, NetworkState};

pub async fn show(State(state): State<NetworkState>) -> Result<Response, ApiError> {
    let shown = state.show.run()?;
    Ok(answer(&state, shown))
}

pub async fn change(
    State(state): State<NetworkState>,
    headers: HeaderMap,
    Json(request): Json<NetworkRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let changed = state.change.run(&request.into_raw(), &revision).await?;
    Ok(answer(&state, changed))
}

fn answer(state: &NetworkState, configured: Revisioned<NetworkSettings>) -> Response {
    let body = NetworkResponse::of(configured.value, state.effective, host_interfaces());
    let mut response = Json(body).into_response();
    response
        .headers_mut()
        .insert(ETAG, configured.revision.etag());
    response
}
