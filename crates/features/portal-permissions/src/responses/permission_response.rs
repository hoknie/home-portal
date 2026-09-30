use serde::Serialize;
use time::OffsetDateTime;

use crate::types::{Advice, Pane, PermissionState, PermissionView};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PermissionResponse {
    pub code: String,
    pub state: PermissionState,
    #[serde(with = "time::serde::rfc3339::option")]
    pub learned_at: Option<OffsetDateTime>,
    pub advice: Option<Advice>,
    pub pane: Pane,
}

impl PermissionResponse {
    pub fn of(view: &PermissionView) -> PermissionResponse {
        PermissionResponse {
            code: view.code.code(),
            state: view.state,
            learned_at: view.learned_at,
            advice: view.advice,
            pane: view.pane(),
        }
    }
}
