use axum::Json;
use axum::extract::{Query, State};
use jiff::Timestamp;
use portal_feature::ApiError;

use crate::parsers::parse_cron;
use crate::requests::ScheduleQuery;
use crate::responses::{CatalogueResponse, ScheduleResponse, ScriptResponse, ScriptsResponse};
use crate::types::{AutomationsState, Trigger};

pub const TIMES: usize = 5;

pub async fn catalogue(State(state): State<AutomationsState>) -> Json<CatalogueResponse> {
    Json(CatalogueResponse::of(
        state.directory.as_ref(),
        &state.sink.cache.webhooks(),
        state.sink.cache.tags(),
    ))
}

pub async fn scripts(State(state): State<AutomationsState>) -> Json<ScriptsResponse> {
    let scripts = state.scripts.clone();
    let listed = tokio::task::spawn_blocking(move || scripts.list())
        .await
        .ok()
        .flatten();
    Json(ScriptsResponse {
        directory: state.scripts.canonical_root().display().to_string(),
        exists: listed.is_some(),
        user_id: crate::clients::effective_user(),
        scripts: listed
            .unwrap_or_default()
            .into_iter()
            .map(ScriptResponse::of)
            .collect(),
    })
}

pub async fn schedule(
    State(state): State<AutomationsState>,
    Query(query): Query<ScheduleQuery>,
) -> Result<Json<ScheduleResponse>, ApiError> {
    let parsed = parse_cron(&query.cron).map_err(|message| ApiError::invalid("cron", &message))?;
    let zone = state.sink.cache.zone();
    let mut cursor = Timestamp::now();
    let mut times = Vec::with_capacity(TIMES);
    for _ in 0..TIMES {
        let Some(next) = parsed.next_after(cursor, &zone) else {
            break;
        };
        times.push(
            next.to_zoned(zone.clone())
                .strftime("%Y-%m-%dT%H:%M:%S%:z")
                .to_string(),
        );
        cursor = next;
    }
    if times.is_empty() {
        return Err(ApiError::invalid("cron", Trigger::NEVER_FIRES));
    }
    Ok(Json(ScheduleResponse {
        timezone: zone.iana_name().unwrap_or("UTC").to_string(),
        times,
    }))
}
