use axum::extract::{Path, State};
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use portal_config::{Revision, Revisioned};
use portal_feature::{ApiError, Principal};

use super::runs::name_of;
use crate::requests::{RunWebhookRequest, WebhookRequest};
use crate::responses::{
    AcceptedResponse, CreatedWebhookResponse, TokenResponse, WebhookResponse, WebhooksResponse,
};
use crate::types::{AutomationsState, WebhookView};

pub async fn list_webhooks(State(state): State<AutomationsState>) -> Response {
    listing(state.list_webhooks.run())
}

pub async fn create_webhook(
    State(state): State<AutomationsState>,
    headers: HeaderMap,
    Json(request): Json<WebhookRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let with_token = request.with_token;
    let created = state
        .create_webhook
        .run(request.into_raw("", None), with_token, &revision)
        .await?;
    let body = CreatedWebhookResponse {
        webhook: response_of(&created.value.view),
        token: created.value.token,
    };
    Ok(with_revision(
        StatusCode::CREATED,
        &created.revision,
        Json(body),
    ))
}

pub async fn update_webhook(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<WebhookRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let changed = state
        .change_webhook
        .run(&id, request.into_raw(&id, None), &revision)
        .await?;
    let body = response_of(&changed.value);
    Ok(with_revision(StatusCode::OK, &changed.revision, Json(body)))
}

pub async fn delete_webhook(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    Ok(listing(state.delete_webhook.run(&id, &revision).await?))
}

pub async fn issue_token(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let issued = state.issue_token.run(&id, &revision).await?;
    let body = TokenResponse {
        token: issued.value,
    };
    Ok(with_revision(StatusCode::OK, &issued.revision, Json(body)))
}

pub async fn remove_token(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    Ok(listing(state.remove_token.run(&id, &revision).await?))
}

pub async fn run_webhook(
    State(state): State<AutomationsState>,
    principal: Option<Extension<Principal>>,
    Path(id): Path<String>,
    Json(request): Json<RunWebhookRequest>,
) -> Result<(StatusCode, Json<AcceptedResponse>), ApiError> {
    let run_id = state
        .run_webhook
        .run(&id, &request.variables, &name_of(principal))?;
    Ok((
        StatusCode::ACCEPTED,
        Json(AcceptedResponse {
            accepted: true,
            run_id: run_id.map(|id| id.to_string()),
        }),
    ))
}

fn response_of(view: &WebhookView) -> WebhookResponse {
    WebhookResponse::of(&view.webhook, view.last)
}

fn listing(views: Revisioned<Vec<WebhookView>>) -> Response {
    let body = WebhooksResponse {
        webhooks: views.value.iter().map(response_of).collect(),
    };
    with_revision(StatusCode::OK, &views.revision, Json(body))
}

fn with_revision(status: StatusCode, revision: &Revision, body: impl IntoResponse) -> Response {
    let mut response = (status, body).into_response();
    response.headers_mut().insert(ETAG, revision.etag());
    response
}
