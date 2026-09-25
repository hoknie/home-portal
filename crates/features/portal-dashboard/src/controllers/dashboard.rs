use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Extension;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::http::header::ETAG;
use axum::response::{IntoResponse, Response};
use portal_config::{ConfigStore, Revision, Snapshot};
use portal_feature::ApiError;
use portal_model::Environment;

use crate::repositories::write_layout;
use crate::requests::{DashboardQuery, LayoutRequest};
use crate::responses::{DashboardResponse, SectionView, WidgetView};
use crate::services::{check_edited, layout, renamed_for_the_editor};

pub const WIDGETS: &str = "dashboard.widgets";
pub const SECTIONS: &str = "dashboard.sections";
pub const DASHBOARD: &str = "dashboard";

pub async fn show(
    State(configuration): State<Arc<ConfigStore>>,
    Extension(environment): Extension<Environment>,
    Query(query): Query<DashboardQuery>,
) -> Result<Response, ApiError> {
    let snapshot = configuration.read();
    let filter = (!query.all).then_some(&environment);
    respond(&snapshot, filter)
}

pub async fn update(
    State(configuration): State<Arc<ConfigStore>>,
    headers: HeaderMap,
    Json(request): Json<LayoutRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let edited = request.into_layout();
    let errors = check_edited(&edited);
    if !errors.is_empty() {
        return Err(ApiError::Invalid(errors));
    }
    let target = target_file(&configuration)?;
    let written = configuration
        .update(&target, &revision, |document| {
            write_layout(document, &edited);
            Ok(())
        })
        .await;
    let (_, snapshot) = written.map_err(|error| match error {
        ApiError::Invalid(errors) => ApiError::Invalid(
            errors
                .into_iter()
                .map(|error| renamed_for_the_editor(error, &edited))
                .collect(),
        ),
        other => other,
    })?;
    respond(&snapshot, None)
}

fn target_file(configuration: &ConfigStore) -> Result<PathBuf, ApiError> {
    let snapshot = configuration.read();
    let origins = &snapshot.origins;
    let files: BTreeSet<PathBuf> = [WIDGETS, SECTIONS]
        .into_iter()
        .flat_map(|section| {
            (0..origins.count(section)).filter_map(move |index| origins.of(section, index))
        })
        .map(PathBuf::from)
        .collect();
    if files.len() > 1 {
        let names: Vec<String> = files
            .iter()
            .map(|path| path.display().to_string())
            .collect();
        return Err(ApiError::Conflict(format!(
            "the layout is spread over {}; move every [[dashboard.widgets]] and [[dashboard.sections]] entry into one file to edit it here",
            names.join(" and ")
        )));
    }
    Ok(files
        .into_iter()
        .next()
        .or_else(|| origins.table(DASHBOARD).map(PathBuf::from))
        .unwrap_or_else(|| configuration.writes_to()))
}

fn respond(snapshot: &Snapshot, filter: Option<&Environment>) -> Result<Response, ApiError> {
    let layout = layout(&snapshot.document)
        .map_err(|message| ApiError::Internal(format!("dashboard section: {message}")))?;
    let widgets = layout
        .widgets
        .into_iter()
        .enumerate()
        .filter(|(_, widget)| filter.is_none_or(|environment| widget.visible_to(environment)))
        .map(|(index, widget)| WidgetView::of(WidgetView::key_of(index, &widget), widget))
        .collect();
    let mut response = Json(DashboardResponse {
        sections: layout.sections.into_iter().map(SectionView::of).collect(),
        widgets,
    })
    .into_response();
    response
        .headers_mut()
        .insert(ETAG, snapshot.revision.etag());
    Ok(response)
}
