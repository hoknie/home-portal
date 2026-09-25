use std::time::{Duration, Instant};

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use portal_feature::{ApiError, Principal};

use super::UNKNOWN_AUTOMATION;
use crate::requests::RunsQuery;
use crate::responses::{QueuedResponse, RunResponse, RunsResponse};
use crate::types::{AutomationsState, RunFilter};

pub const DISABLED: &str = "the automation is disabled";
pub const MANUAL_EVERY: Duration = Duration::from_secs(5);

pub async fn run_now(
    State(state): State<AutomationsState>,
    principal: Option<Extension<Principal>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<QueuedResponse>), ApiError> {
    let automation = state
        .sink
        .cache
        .find(&id)
        .ok_or(ApiError::NotFound(UNKNOWN_AUTOMATION))?;
    if !automation.enabled {
        return Err(ApiError::Conflict(DISABLED.to_string()));
    }
    {
        let mut manual = state
            .manual_runs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = Instant::now();
        if let Some(last) = manual.get(&id) {
            let since = now.saturating_duration_since(*last);
            if since < MANUAL_EVERY {
                return Err(ApiError::TooManyRequests {
                    retry_after_seconds: (MANUAL_EVERY - since).as_secs().max(1),
                });
            }
        }
        manual.insert(id.clone(), now);
    }
    let by = principal
        .map(|Extension(principal)| principal.name)
        .unwrap_or_default();
    let run_id = state.sink.run_now(&automation, &by);
    Ok((
        StatusCode::ACCEPTED,
        Json(QueuedResponse {
            run_id: run_id.to_string(),
        }),
    ))
}

pub async fn runs(
    State(state): State<AutomationsState>,
    Query(query): Query<RunsQuery>,
) -> Json<RunsResponse> {
    Json(RunsResponse {
        runs: state
            .sink
            .journal
            .matching(&RunFilter {
                automation: query.automation,
                webhook: query.webhook,
                text: query.text,
            })
            .iter()
            .map(RunResponse::of)
            .collect(),
    })
}
