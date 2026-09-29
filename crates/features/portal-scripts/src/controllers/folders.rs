use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use portal_feature::{ApiError, Principal};
use portal_model::DetectedEnvironment;

use super::access::{blocking, logged, name_of, writable};
use crate::requests::{FolderQuery, FolderRequest};
use crate::types::ScriptsState;

pub async fn create_folder(
    State(state): State<ScriptsState>,
    (principal, detected): (
        Option<Extension<Principal>>,
        Option<Extension<DetectedEnvironment>>,
    ),
    request: Option<Json<FolderRequest>>,
) -> Result<Response, ApiError> {
    writable(&state, detected.as_ref())?;
    let name = request.and_then(|Json(request)| request.name);
    let create = state.create_folder.clone();
    let named = name.clone();
    blocking(move || create.run(named.as_deref())).await?;
    logged(
        "folder created",
        &name_of(principal),
        name.as_deref().unwrap_or("."),
    );
    Ok(StatusCode::CREATED.into_response())
}

pub async fn remove_folder(
    State(state): State<ScriptsState>,
    (principal, detected): (
        Option<Extension<Principal>>,
        Option<Extension<DetectedEnvironment>>,
    ),
    Query(query): Query<FolderQuery>,
) -> Result<Response, ApiError> {
    writable(&state, detected.as_ref())?;
    let delete = state.delete_folder.clone();
    let name = query.name.clone();
    blocking(move || delete.run(&name)).await?;
    logged("folder deleted", &name_of(principal), &query.name);
    Ok(StatusCode::NO_CONTENT.into_response())
}
