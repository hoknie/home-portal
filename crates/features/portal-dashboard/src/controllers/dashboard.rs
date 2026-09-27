use axum::Extension;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::http::header::ETAG;
use axum::response::{IntoResponse, Response};
use portal_config::{Revision, Revisioned};
use portal_feature::ApiError;
use portal_model::Environment;

use crate::requests::{DashboardQuery, LayoutRequest};
use crate::responses::{DashboardResponse, SectionView, WidgetView};
use crate::types::{DashboardState, LayoutView};

pub async fn show(
    State(state): State<DashboardState>,
    Extension(environment): Extension<Environment>,
    Query(query): Query<DashboardQuery>,
) -> Result<Response, ApiError> {
    let filter = (!query.all).then_some(&environment);
    Ok(respond(state.show.run(filter)?))
}

pub async fn update(
    State(state): State<DashboardState>,
    headers: HeaderMap,
    Json(request): Json<LayoutRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let changed = state.change.run(&request.into_layout(), &revision).await?;
    Ok(respond(changed))
}

fn respond(view: Revisioned<LayoutView>) -> Response {
    let layout = view.value;
    let widgets = layout
        .widgets
        .into_iter()
        .map(|(index, widget)| WidgetView::of(WidgetView::key_of(index, &widget), widget))
        .collect();
    let mut response = Json(DashboardResponse {
        sections: layout.sections.into_iter().map(SectionView::of).collect(),
        widgets,
    })
    .into_response();
    response.headers_mut().insert(ETAG, view.revision.etag());
    response
}
