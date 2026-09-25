use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use portal_config::Revision;
use portal_feature::ApiError;

use super::proxy::answer;
use crate::clients::CaddyAdmin;
use crate::repositories::{SECTION, write_caddy_source, write_managed};
use crate::requests::CaddySourceRequest;
use crate::types::ProxyState;

pub const NOT_INSTALLED: &str = "Caddy is not downloaded yet";

pub async fn download(State(state): State<ProxyState>) -> Result<Response, ApiError> {
    let source = state.sync.settings().caddy;
    state
        .sync
        .manager()
        .begin_download(source)
        .map_err(|message| ApiError::Conflict(message.to_string()))?;
    Ok(answer(&state, StatusCode::ACCEPTED))
}

pub async fn start(
    State(state): State<ProxyState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    if state.sync.manager().installed().is_none() {
        return Err(ApiError::Conflict(NOT_INSTALLED.to_string()));
    }
    write(&state, &headers, true).await?;
    let settings = state.sync.settings();
    let admin = CaddyAdmin::new(&settings.admin).map_err(ApiError::Internal)?;
    if admin.config().await.is_err() && !CaddyAdmin::occupied(&settings.admin).await {
        state
            .sync
            .manager()
            .launch(&settings.admin)
            .map_err(ApiError::Internal)?;
    }
    Ok(answer(&state, StatusCode::OK))
}

pub async fn stop(
    State(state): State<ProxyState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    write(&state, &headers, false).await?;
    let settings = state.sync.settings();
    let admin = CaddyAdmin::new(&settings.admin).map_err(ApiError::Internal)?;
    if let Err(problem) = admin.stop().await {
        tracing::info!(%problem, "caddy was not running");
    }
    Ok(answer(&state, StatusCode::OK))
}

pub async fn change_source(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(request): Json<CaddySourceRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let source = request.into_source().map_err(ApiError::Invalid)?;
    let target = target(&state);
    state
        .configuration
        .update(&target, &revision, |document| {
            write_caddy_source(document, &source);
            Ok(())
        })
        .await?;
    Ok(answer(&state, StatusCode::OK))
}

fn target(state: &ProxyState) -> std::path::PathBuf {
    state
        .configuration
        .read()
        .origins
        .table(SECTION)
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| state.configuration.writes_to())
}

async fn write(state: &ProxyState, headers: &HeaderMap, managed: bool) -> Result<(), ApiError> {
    let revision = Revision::from_headers(headers)?;
    let target = target(state);
    state
        .configuration
        .update(&target, &revision, |document| {
            write_managed(document, managed);
            Ok(())
        })
        .await?;
    Ok(())
}
