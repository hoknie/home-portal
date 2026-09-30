use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use super::bounded::run_bounded;
use crate::ports::PermissionCheck;
use crate::types::{Advice, Finding, PermissionCode, Ran};

pub const OSASCRIPT: &str = "/usr/bin/osascript";
pub const SCRIPT: [&str; 3] = [
    "on run argv",
    "tell application (item 1 of argv) to get name",
    "end run",
];
pub const NOT_AUTHORIZED: &str = "(-1743)";
pub const NOT_FOUND: [&str; 3] = ["(-600)", "(-1728)", "(-10814)"];

pub struct AutomationCheck {
    pub application: String,
    pub program: PathBuf,
    pub limit: Duration,
}

impl PermissionCheck for AutomationCheck {
    fn code(&self) -> PermissionCode {
        PermissionCode::Automation(self.application.clone())
    }

    fn ask(&self) -> Finding {
        let mut command = Command::new(&self.program);
        for line in SCRIPT {
            command.arg("-e").arg(line);
        }
        command.arg(&self.application);
        match run_bounded(command, self.limit) {
            Ran::Finished(output) if output.status.success() => Finding::granted(),
            Ran::Finished(output) => refusal(&String::from_utf8_lossy(&output.stderr)),
            Ran::TimedOut | Ran::NotStarted => Finding::unknown(Advice::CheckFailed),
        }
    }
}

pub fn refusal(error: &str) -> Finding {
    if error.contains(NOT_AUTHORIZED) {
        Finding::denied(Advice::AllowInSettings)
    } else if NOT_FOUND.iter().any(|code| error.contains(code)) {
        Finding::unknown(Advice::ApplicationNotFound)
    } else {
        Finding::unknown(Advice::CheckFailed)
    }
}
