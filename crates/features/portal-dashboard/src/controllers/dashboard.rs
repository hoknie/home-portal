use axum::Extension;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::http::header::ETAG;
use axum::response::{IntoResponse, Response};
use portal_config::{Revision, Revisioned};
use portal_feature::{Action, ApiError, Area, Principal, Right};
use portal_model::{DetectedEnvironment, Environment};

use crate::requests::{DashboardQuery, LayoutRequest};
use crate::responses::{DashboardResponse, SectionView, WidgetView};
use crate::types::{DashboardState, LayoutView};

pub const EDIT_LAYOUT: Right = Right::new(Area::Layout, Action::Update);

pub async fn show(
    State(state): State<DashboardState>,
    Extension(environment): Extension<Environment>,
    principal: Option<Extension<Principal>>,
    detected: Option<Extension<DetectedEnvironment>>,
    Query(query): Query<DashboardQuery>,
) -> Result<Response, ApiError> {
    let editor = principal.is_some_and(|Extension(principal)| principal.rights.allows(EDIT_LAYOUT));
    let inside = detected.is_some_and(|Extension(detected)| !detected.environment.is_internet());
    let whole = query.all && editor && inside;
    let filter = (!whole).then_some(&environment);
    Ok(respond(state.show.run(filter)?))
}

pub async fn update(
    State(state): State<DashboardState>,
    principal: Option<Extension<Principal>>,
    headers: HeaderMap,
    Json(request): Json<LayoutRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let rights = principal
        .map(|Extension(principal)| principal.rights)
        .unwrap_or_default();
    let changed = state
        .change
        .run(&request.into_layout(), (&revision, &rights))
        .await?;
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
