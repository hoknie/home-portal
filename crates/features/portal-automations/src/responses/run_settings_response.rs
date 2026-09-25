use serde::Serialize;

use crate::types::RunSettings;

#[derive(Debug, Clone, Serialize)]
pub struct RunSettingsResponse {
    pub script: String,
    pub args: Vec<String>,
    pub timeout_seconds: u64,
}

impl RunSettingsResponse {
    pub fn of(run: &RunSettings) -> RunSettingsResponse {
        RunSettingsResponse {
            script: run.script.clone(),
            args: run.args.clone(),
            timeout_seconds: run.timeout_seconds,
        }
    }
}
