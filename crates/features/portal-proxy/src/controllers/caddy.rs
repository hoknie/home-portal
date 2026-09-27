use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use portal_config::Revision;
use portal_feature::ApiError;

use super::proxy::answer;
use crate::requests::CaddySourceRequest;
use crate::types::ProxyState;

pub async fn download(State(state): State<ProxyState>) -> Result<Response, ApiError> {
    Ok(answer(state.download.run()?, StatusCode::ACCEPTED))
}

pub async fn start(
    State(state): State<ProxyState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let started = state.start.run(Revision::from_headers(&headers)).await?;
    Ok(answer(started, StatusCode::OK))
}

pub async fn stop(
    State(state): State<ProxyState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    Ok(answer(state.stop.run(&revision).await?, StatusCode::OK))
}

pub async fn change_source(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(request): Json<CaddySourceRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let source = request.into_source().map_err(ApiError::Invalid)?;
    let changed = state.change_source.run(&source, &revision).await?;
    Ok(answer(changed, StatusCode::OK))
}
