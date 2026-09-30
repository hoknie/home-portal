use crate::ports::PermissionCheck;
use crate::types::{Finding, PermissionCode};

pub struct NotApplicableCheck {
    pub code: PermissionCode,
}

impl PermissionCheck for NotApplicableCheck {
    fn code(&self) -> PermissionCode {
        self.code.clone()
    }

    fn ask(&self) -> Finding {
        Finding::not_applicable(None)
    }
}
