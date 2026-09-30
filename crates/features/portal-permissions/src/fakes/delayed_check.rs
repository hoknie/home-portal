use std::thread;
use std::time::Duration;

use crate::ports::PermissionCheck;
use crate::types::{Finding, PermissionCode};

pub struct DelayedCheck {
    pub code: PermissionCode,
    pub finding: Finding,
    pub delay: Duration,
}

impl PermissionCheck for DelayedCheck {
    fn code(&self) -> PermissionCode {
        self.code.clone()
    }

    fn ask(&self) -> Finding {
        thread::sleep(self.delay);
        self.finding
    }
}
