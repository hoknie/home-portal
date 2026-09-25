use std::path::PathBuf;
use std::time::Duration;

use serde_json::{Map, Value};

use super::Pending;
use crate::helpers::render;
use crate::types::Catalogue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub program: PathBuf,
    pub directory: PathBuf,
    pub arguments: Vec<String>,
    pub environment: Vec<(String, String)>,
    pub input: String,
    pub timeout: Duration,
}

impl Invocation {
    pub const PASSED_THROUGH: [&'static str; 4] = ["PATH", "HOME", "LANG", "TZ"];
    pub const VARIABLE_PREFIX: &'static str = "PORTAL_";

    pub fn for_run(pending: &Pending, program: PathBuf, directory: PathBuf) -> Invocation {
        let fields = Self::fields_of(pending);
        let lookup = |name: &str| {
            fields
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.as_str())
        };
        let arguments = pending
            .automation
            .run
            .args
            .iter()
            .map(|template| render(template, lookup))
            .collect();
        let mut environment: Vec<(String, String)> = Self::PASSED_THROUGH
            .iter()
            .filter_map(|name| {
                std::env::var(name)
                    .ok()
                    .map(|value| (name.to_string(), value))
            })
            .collect();
        environment.extend(
            fields
                .iter()
                .map(|(key, value)| (Self::variable_of(key), value.clone())),
        );
        let input: Map<String, Value> = fields
            .iter()
            .map(|(key, value)| (key.clone(), Value::String(value.clone())))
            .collect();
        Invocation {
            program,
            directory,
            arguments,
            environment,
            input: Value::Object(input).to_string(),
            timeout: Duration::from_secs(pending.automation.run.timeout_seconds),
        }
    }

    pub fn fields_of(pending: &Pending) -> Vec<(String, String)> {
        let mut fields: Vec<(String, String)> = pending
            .event
            .fields
            .iter()
            .map(|(key, value)| (key.to_string(), value.clone()))
            .chain(pending.event.variables.iter().cloned())
            .collect();
        fields.push((
            Catalogue::AUTOMATION_FIELD.to_string(),
            pending.automation.id.clone(),
        ));
        fields.push((
            Catalogue::RUN_ID_FIELD.to_string(),
            pending.run_id.to_string(),
        ));
        fields.push((
            Catalogue::RUN_MANUAL_FIELD.to_string(),
            pending.manual().to_string(),
        ));
        fields.push((
            Catalogue::RUN_BY_FIELD.to_string(),
            pending.by.clone().unwrap_or_default(),
        ));
        fields
    }

    pub fn variable_of(field: &str) -> String {
        format!(
            "{}{}",
            Self::VARIABLE_PREFIX,
            field.to_ascii_uppercase().replace('.', "_")
        )
    }
}
