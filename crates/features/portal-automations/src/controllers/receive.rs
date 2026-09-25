use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Instant;

use axum::body::Bytes;
use axum::extract::rejection::QueryRejection;
use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use portal_feature::{ApiError, ClientAddress, EventName, EventSink, FieldError, PortalEvent};
use serde_json::Value;
use time::OffsetDateTime;

use crate::helpers::{same_secret, token_hash};
use crate::responses::AcceptedResponse;
use crate::types::{AutomationsState, Webhook, WebhookAction};

pub const UNKNOWN_WEBHOOK: &str = "no such webhook";
pub const TOKEN_HEADER: &str = "x-webhook-token";
pub const BEARER: &str = "Bearer ";
pub const STOPPING: &str = "the portal is stopping";

pub async fn receive(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    connect: Option<Extension<ConnectInfo<SocketAddr>>>,
    client: Option<Extension<ClientAddress>>,
    query: Result<Query<HashMap<String, String>>, QueryRejection>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<AcceptedResponse>), ApiError> {
    let Query(query) = query.map_err(|rejection| ApiError::BadRequest(rejection.body_text()))?;
    let client = client
        .map(|Extension(ClientAddress(address))| address.to_string())
        .or_else(|| connect.map(|Extension(ConnectInfo(address))| address.ip().to_string()))
        .unwrap_or_default();
    let webhook = state
        .sink
        .cache
        .webhook(&id)
        .filter(|webhook| webhook.enabled)
        .ok_or(ApiError::NotFound(UNKNOWN_WEBHOOK))?;
    if let Some(expected) = &webhook.token_sha256 {
        let presented = presented_token(&headers).ok_or(ApiError::Unauthorized)?;
        if !same_secret(&token_hash(&presented), expected) {
            return Err(ApiError::Unauthorized);
        }
    }
    if state.sink.stopping() {
        return Err(ApiError::ServiceUnavailable(STOPPING.to_string()));
    }
    let answer = accept(&state, &webhook, (&query, &body), &client);
    let status = match &answer {
        Ok((status, _)) => status.as_u16(),
        Err(error) => error.status().as_u16(),
    };
    state.webhooks.record(&webhook.id, status);
    answer
}

type Request<'a> = (&'a HashMap<String, String>, &'a Bytes);

fn accept(
    state: &AutomationsState,
    webhook: &Webhook,
    (query, body): Request<'_>,
    client: &str,
) -> Result<(StatusCode, Json<AcceptedResponse>), ApiError> {
    state
        .webhooks
        .allow(&webhook.id, Instant::now())
        .map_err(|retry_after_seconds| ApiError::TooManyRequests {
            retry_after_seconds,
        })?;
    let variables = variables_of(webhook, query, body)?;
    let event = PortalEvent::of(
        EventName::WebhookReceived,
        OffsetDateTime::now_utc(),
        &[
            ("webhook.id", webhook.id.as_str()),
            ("webhook.title", webhook.title.as_str()),
            ("client.address", client),
        ],
    )
    .with_variables(&variables);
    let run_id = match webhook.action {
        WebhookAction::Event => {
            state.sink.emit(event);
            None
        }
        WebhookAction::Script(_) => state.sink.run_webhook(webhook, event),
    };
    Ok((
        StatusCode::ACCEPTED,
        Json(AcceptedResponse {
            accepted: true,
            run_id: run_id.map(|id| id.to_string()),
        }),
    ))
}

fn presented_token(headers: &HeaderMap) -> Option<String> {
    let header = |name: &str| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
    };
    header(axum::http::header::AUTHORIZATION.as_str())
        .and_then(|value| value.strip_prefix(BEARER))
        .or_else(|| header(TOKEN_HEADER))
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
}

fn variables_of(
    webhook: &Webhook,
    query: &HashMap<String, String>,
    body: &Bytes,
) -> Result<Vec<(String, String)>, ApiError> {
    let object = serde_json::from_slice::<Value>(body)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    let mut found = Vec::new();
    let mut missing = Vec::new();
    for name in &webhook.variables {
        let value = object
            .get(name)
            .map(|value| match value {
                Value::String(text) => text.clone(),
                other => other.to_string(),
            })
            .or_else(|| query.get(name).cloned());
        match value {
            Some(value) => found.push((name.clone(), value)),
            None => missing.push(FieldError::new(name.clone(), "is required")),
        }
    }
    if missing.is_empty() {
        Ok(found)
    } else {
        Err(ApiError::Invalid(missing))
    }
}
