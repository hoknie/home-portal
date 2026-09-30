use std::path::PathBuf;
use std::time::Duration;

use super::{EventValues, Pending};
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
        let event = Self::values_of(pending);
        let arguments = pending
            .automation
            .run
            .args
            .iter()
            .map(|template| event.render(template))
            .collect();
        Self::for_step(
            (program, directory),
            arguments,
            &event,
            Duration::from_secs(pending.automation.run.timeout_seconds),
        )
    }

    pub fn for_step(
        (program, directory): (PathBuf, PathBuf),
        arguments: Vec<String>,
        event: &EventValues,
        timeout: Duration,
    ) -> Invocation {
        let mut environment: Vec<(String, String)> = Self::PASSED_THROUGH
            .iter()
            .filter_map(|name| {
                std::env::var(name)
                    .ok()
                    .map(|value| (name.to_string(), value))
            })
            .collect();
        environment.extend(
            event
                .fields
                .iter()
                .map(|(key, value)| (Self::variable_of(key), value.clone())),
        );
        Invocation {
            program,
            directory,
            arguments,
            environment,
            input: event.input(),
            timeout,
        }
    }

    pub fn values_of(pending: &Pending) -> EventValues {
        EventValues::new(Self::fields_of(pending), pending.event.body.clone())
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
