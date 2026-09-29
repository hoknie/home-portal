use axum::extract::{Query, State};
use axum::response::Response;
use axum::{Extension, Json};
use portal_feature::ApiError;
use portal_model::DetectedEnvironment;

use super::access::{blocking, inside, switched_on, tagged};
use crate::requests::PathQuery;
use crate::responses::{ScriptFileResponse, ScriptTextResponse, ScriptTreeResponse};
use crate::types::ScriptsState;

pub async fn tree(
    State(state): State<ScriptsState>,
    detected: Option<Extension<DetectedEnvironment>>,
) -> Result<Json<ScriptTreeResponse>, ApiError> {
    switched_on(&state)?;
    let list = state.list.clone();
    let listed = blocking(move || Ok(list.tree())).await?;
    Ok(Json(ScriptTreeResponse {
        directory: state.list.root().display().to_string(),
        exists: listed.is_some(),
        user_id: state.list.user(),
        inside: inside(detected.as_ref()),
        left_out: listed.as_ref().map_or(0, |tree| tree.left_out),
        folders: listed
            .as_ref()
            .map(|tree| tree.folders.clone())
            .unwrap_or_default(),
        files: listed
            .map(|tree| tree.files.iter().map(ScriptFileResponse::of).collect())
            .unwrap_or_default(),
    }))
}

pub async fn read(
    State(state): State<ScriptsState>,
    Query(query): Query<PathQuery>,
) -> Result<Response, ApiError> {
    switched_on(&state)?;
    let read = state.read.clone();
    let path = query.path.clone();
    let (content, revision, entry) = blocking(move || read.run(&path)).await?;
    Ok(tagged(
        &revision,
        Json(ScriptTextResponse {
            path: entry.path.clone(),
            content,
            revision: revision.to_string(),
            entry: ScriptFileResponse::of(&entry),
        }),
    ))
}
