use std::collections::HashSet;
use std::time::{Duration, Instant};

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use portal_feature::{ApiError, Principal};

use super::UNKNOWN_AUTOMATION;
use crate::requests::RunsQuery;
use crate::responses::{QueuedResponse, RunResponse, RunsResponse};
use crate::types::{AutomationsState, RunFilter, StopAnswer};

pub const DISABLED: &str = "the automation is disabled";
pub const UNKNOWN_RUN: &str = "no such run";
pub const ALREADY_FINISHED: &str = "the run has already finished";
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
    let run_id = state.sink.run_now(&automation, &name_of(principal));
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
    let filter = RunFilter {
        automation: query.automation,
        webhook: query.webhook,
        text: query.text,
    };
    let finished = state.sink.journal.matching(&filter);
    let recorded: HashSet<u64> = finished.iter().map(|record| record.id).collect();
    let active = state
        .sink
        .active
        .matching(&filter)
        .into_iter()
        .filter(|run| !recorded.contains(&run.run_id));
    Json(RunsResponse {
        runs: active
            .map(|run| RunResponse::active(&run))
            .chain(finished.iter().map(RunResponse::of))
            .collect(),
    })
}

pub async fn run(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
) -> Result<Json<RunResponse>, ApiError> {
    current(&state, run_id(&id)?).map(Json)
}

pub async fn stop(
    State(state): State<AutomationsState>,
    principal: Option<Extension<Principal>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<RunResponse>), ApiError> {
    let run_id = run_id(&id)?;
    match state.sink.stop(run_id, &name_of(principal)) {
        StopAnswer::Stopping | StopAnswer::AlreadyStopping => {
            Ok((StatusCode::ACCEPTED, Json(current(&state, run_id)?)))
        }
        StopAnswer::Finished => Err(ApiError::Conflict(ALREADY_FINISHED.to_string())),
        StopAnswer::Unknown => Err(ApiError::NotFound(UNKNOWN_RUN)),
    }
}

fn current(state: &AutomationsState, run_id: u64) -> Result<RunResponse, ApiError> {
    if let Some(record) = state.sink.journal.find(run_id) {
        return Ok(RunResponse::of(&record));
    }
    state
        .sink
        .active
        .find(run_id)
        .map(|run| RunResponse::active(&run))
        .ok_or(ApiError::NotFound(UNKNOWN_RUN))
}

fn run_id(id: &str) -> Result<u64, ApiError> {
    id.parse().map_err(|_| ApiError::NotFound(UNKNOWN_RUN))
}

fn name_of(principal: Option<Extension<Principal>>) -> String {
    principal
        .map(|Extension(principal)| principal.name)
        .unwrap_or_default()
}
