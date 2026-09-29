use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use portal_config::Revision;
use portal_feature::{ApiError, Principal};
use portal_model::DetectedEnvironment;

use super::access::{blocking, logged, name_of, tagged, writable};
use crate::requests::{CreateRequest, MoveRequest, PathQuery, WriteRequest};
use crate::responses::ScriptFileResponse;
use crate::types::ScriptsState;

pub async fn replace(
    State(state): State<ScriptsState>,
    (principal, detected): (
        Option<Extension<Principal>>,
        Option<Extension<DetectedEnvironment>>,
    ),
    headers: HeaderMap,
    Query(query): Query<PathQuery>,
    Json(request): Json<WriteRequest>,
) -> Result<Response, ApiError> {
    writable(&state, detected.as_ref())?;
    let expected = Revision::from_headers(&headers)?;
    let write = state.write.clone();
    let path = query.path.clone();
    let (revision, entry) = blocking(move || write.run(&path, &request.content, &expected)).await?;
    logged("written", &name_of(principal), &entry.path);
    Ok(tagged(&revision, Json(ScriptFileResponse::of(&entry))))
}

pub async fn create(
    State(state): State<ScriptsState>,
    (principal, detected): (
        Option<Extension<Principal>>,
        Option<Extension<DetectedEnvironment>>,
    ),
    Json(request): Json<CreateRequest>,
) -> Result<Response, ApiError> {
    writable(&state, detected.as_ref())?;
    let create = state.create.clone();
    let (revision, entry) = blocking(move || create.run(&request.path, &request.content)).await?;
    logged("created", &name_of(principal), &entry.path);
    Ok(tagged(
        &revision,
        (StatusCode::CREATED, Json(ScriptFileResponse::of(&entry))),
    ))
}

pub async fn remove(
    State(state): State<ScriptsState>,
    (principal, detected): (
        Option<Extension<Principal>>,
        Option<Extension<DetectedEnvironment>>,
    ),
    headers: HeaderMap,
    Query(query): Query<PathQuery>,
) -> Result<Response, ApiError> {
    writable(&state, detected.as_ref())?;
    let expected = Revision::from_headers(&headers)?;
    let delete = state.delete.clone();
    let path = query.path.clone();
    blocking(move || delete.run(&path, &expected)).await?;
    logged("deleted", &name_of(principal), &query.path);
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub async fn move_script(
    State(state): State<ScriptsState>,
    (principal, detected): (
        Option<Extension<Principal>>,
        Option<Extension<DetectedEnvironment>>,
    ),
    headers: HeaderMap,
    Json(request): Json<MoveRequest>,
) -> Result<Response, ApiError> {
    writable(&state, detected.as_ref())?;
    let expected = Revision::from_headers(&headers)?;
    let move_script = state.move_script.clone();
    let from = request.from.clone();
    let (revision, entry) =
        blocking(move || move_script.run(&from, &request.to, &expected)).await?;
    logged(
        "moved",
        &name_of(principal),
        &format!("{} -> {}", request.from, entry.path),
    );
    Ok(tagged(&revision, Json(ScriptFileResponse::of(&entry))))
}
