use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use portal_feature::ApiError;
use portal_model::Environment;
use time::OffsetDateTime;

use crate::requests::HistoryQuery;
use crate::responses::HistoryResponse;
use crate::types::{HistoryRange, ServicesSection, ServicesState};

pub const UNKNOWN_SERVICE: &str = "no such service";
pub const UNKNOWN_RANGE: &str = "range must be one of 24h, 7d and 30d";

pub async fn history(
    State(state): State<ServicesState>,
    Extension(environment): Extension<Environment>,
    Path(id): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>, ApiError> {
    let range = match query.range.as_deref() {
        None => HistoryRange::Day,
        Some(name) => HistoryRange::parse(name)
            .ok_or_else(|| ApiError::BadRequest(UNKNOWN_RANGE.to_string()))?,
    };
    let document = state.configuration.read().document;
    if ServicesSection::visible(&document, &id, &environment).is_none() {
        return Err(ApiError::NotFound(UNKNOWN_SERVICE));
    }
    let view = state.board.history(&id, range, OffsetDateTime::now_utc());
    Ok(Json(HistoryResponse::of(&view)))
}
