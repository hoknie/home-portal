use axum::Json;
use axum::extract::{Path, State};
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use portal_config::{Revision, Snapshot};
use portal_feature::ApiError;

use super::UNKNOWN_WEBHOOK;
use crate::helpers::{new_token, new_webhook_id, token_hash};
use crate::repositories::{
    append_webhook, remove_webhook, replace_webhook, set_token, webhook_origin, webhook_position,
};
use crate::requests::WebhookRequest;
use crate::responses::{CreatedWebhookResponse, TokenResponse, WebhookResponse, WebhooksResponse};
use crate::services::decoded_webhooks;
use crate::types::{AutomationsSection, AutomationsState, Webhook};

pub async fn list_webhooks(State(state): State<AutomationsState>) -> Response {
    let snapshot = state.configuration.read();
    let body = listed(&state, &snapshot);
    with_revision(StatusCode::OK, &snapshot, Json(body))
}

pub async fn create_webhook(
    State(state): State<AutomationsState>,
    headers: HeaderMap,
    Json(request): Json<WebhookRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let token = request.with_token.then(new_token);
    let raw = request.into_raw(&new_webhook_id(), token.as_deref().map(token_hash));
    let webhook = Webhook::decode(&raw).map_err(ApiError::Invalid)?;
    let target = state.configuration.writes_to();
    let (_, snapshot) = state
        .configuration
        .update(&target, &revision, |document| {
            append_webhook(document, &webhook);
            Ok(())
        })
        .await?;
    state.sink.cache.refresh(&snapshot.document);
    let body = CreatedWebhookResponse {
        webhook: WebhookResponse::of(&webhook, None),
        token,
    };
    Ok(with_revision(StatusCode::CREATED, &snapshot, Json(body)))
}

pub async fn update_webhook(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<WebhookRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let token_sha256 = AutomationsSection::read(&state.configuration.read().document)
        .ok()
        .and_then(|section| section.webhooks.into_iter().find(|raw| raw.id == id))
        .ok_or(ApiError::NotFound(UNKNOWN_WEBHOOK))?
        .token_sha256;
    let webhook =
        Webhook::decode(&request.into_raw(&id, token_sha256)).map_err(ApiError::Invalid)?;
    let snapshot = write(&state, &id, &revision, |document, index| {
        replace_webhook(document, index, &webhook)
    })
    .await?;
    let body = WebhookResponse::of(&webhook, state.webhooks.last(&id));
    Ok(with_revision(StatusCode::OK, &snapshot, Json(body)))
}

pub async fn delete_webhook(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let snapshot = write(&state, &id, &revision, remove_webhook).await?;
    let body = listed(&state, &snapshot);
    Ok(with_revision(StatusCode::OK, &snapshot, Json(body)))
}

pub async fn issue_token(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let token = new_token();
    let hash = token_hash(&token);
    let snapshot = write(&state, &id, &revision, |document, index| {
        set_token(document, index, Some(&hash))
    })
    .await?;
    Ok(with_revision(
        StatusCode::OK,
        &snapshot,
        Json(TokenResponse { token }),
    ))
}

pub async fn remove_token(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let snapshot = write(&state, &id, &revision, |document, index| {
        set_token(document, index, None)
    })
    .await?;
    let body = listed(&state, &snapshot);
    Ok(with_revision(StatusCode::OK, &snapshot, Json(body)))
}

async fn write(
    state: &AutomationsState,
    id: &str,
    revision: &Revision,
    change: impl FnOnce(&mut toml_edit::DocumentMut, usize),
) -> Result<Snapshot, ApiError> {
    let target = webhook_origin(&state.configuration.read(), id)
        .ok_or(ApiError::NotFound(UNKNOWN_WEBHOOK))?;
    let (_, snapshot) = state
        .configuration
        .update(&target, revision, |document| {
            let index =
                webhook_position(document, id).ok_or(ApiError::NotFound(UNKNOWN_WEBHOOK))?;
            change(document, index);
            Ok(())
        })
        .await?;
    state.sink.cache.refresh(&snapshot.document);
    Ok(snapshot)
}

fn listed(state: &AutomationsState, snapshot: &Snapshot) -> WebhooksResponse {
    WebhooksResponse {
        webhooks: decoded_webhooks(&snapshot.document)
            .iter()
            .map(|webhook| WebhookResponse::of(webhook, state.webhooks.last(&webhook.id)))
            .collect(),
    }
}

fn with_revision(status: StatusCode, snapshot: &Snapshot, body: impl IntoResponse) -> Response {
    let mut response = (status, body).into_response();
    response
        .headers_mut()
        .insert(ETAG, snapshot.revision.etag());
    response
}
