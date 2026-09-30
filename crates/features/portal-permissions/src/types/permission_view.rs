use time::OffsetDateTime;

use super::{Advice, Pane, PermissionCode, PermissionState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionView {
    pub code: PermissionCode,
    pub state: PermissionState,
    pub advice: Option<Advice>,
    pub learned_at: Option<OffsetDateTime>,
}

impl PermissionView {
    pub fn pane(&self) -> Pane {
        self.code.pane()
    }
}
