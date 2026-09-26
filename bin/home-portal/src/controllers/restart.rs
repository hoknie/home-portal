use axum::Extension;
use axum::extract::State;
use axum::http::StatusCode;
use portal_feature::Principal;

use crate::types::Restart;

pub const RESTART_PATH: &str = "/api/portal/restart";

pub async fn restart(
    State(restart): State<Restart>,
    principal: Option<Extension<Principal>>,
) -> StatusCode {
    let by = principal
        .map(|Extension(principal)| principal.name)
        .unwrap_or_default();
    tracing::info!(by, "a restart of the portal was asked for");
    restart.request();
    StatusCode::ACCEPTED
}
