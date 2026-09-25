use axum::extract::{Query, State};
use axum::response::Redirect;

use crate::helpers::continuation;
use crate::requests::ContinueQuery;
use crate::types::ProxyState;

pub async fn resume(
    State(state): State<ProxyState>,
    Query(query): Query<ContinueQuery>,
) -> Redirect {
    Redirect::to(&continuation(
        query.to.as_deref(),
        &state.sync.known_hosts(),
        state.sync.settings().https_port,
    ))
}
