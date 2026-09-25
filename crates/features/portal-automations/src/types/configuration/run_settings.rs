use portal_feature::FieldError;

use super::RawRun;
use crate::helpers::script_shape_problem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSettings {
    pub script: String,
    pub args: Vec<String>,
    pub timeout_seconds: u64,
}

impl RunSettings {
    pub const DEFAULT_TIMEOUT: u64 = 60;
    pub const LONGEST_TIMEOUT: u64 = 3600;

    pub fn decode(raw: &RawRun) -> Result<RunSettings, Vec<FieldError>> {
        let mut errors = Vec::new();
        if let Some(problem) = script_shape_problem(&raw.script) {
            errors.push(FieldError::new("run.script", problem));
        }
        let timeout = raw.timeout_seconds.unwrap_or(Self::DEFAULT_TIMEOUT as i64);
        if !(1..=Self::LONGEST_TIMEOUT as i64).contains(&timeout) {
            errors.push(FieldError::new(
                "run.timeout_seconds",
                format!("must be 1 to {}", Self::LONGEST_TIMEOUT),
            ));
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(RunSettings {
            script: raw.script.clone(),
            args: raw.args.clone(),
            timeout_seconds: timeout as u64,
        })
    }
}
