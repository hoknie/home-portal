use portal_config::ConfigError;

use crate::types::BootError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub file: Option<String>,
    pub field: Option<String>,
    pub message: String,
}

impl Problem {
    fn whole(file: Option<String>, message: String) -> Problem {
        Problem {
            file,
            field: None,
            message,
        }
    }
}

pub fn problems_of(error: &BootError) -> Vec<Problem> {
    match error {
        BootError::Configuration(error) => configuration_problems(error),
        other => vec![Problem::whole(None, other.to_string())],
    }
}

fn configuration_problems(error: &ConfigError) -> Vec<Problem> {
    let file = match error {
        ConfigError::Missing { path, .. }
        | ConfigError::Unreadable { path, .. }
        | ConfigError::Syntax { path, .. }
        | ConfigError::Permissions { path, .. }
        | ConfigError::Invalid { path, .. } => Some(path.display().to_string()),
        ConfigError::NoConfigurationPath | ConfigError::Merge { .. } => None,
    };
    match error {
        ConfigError::Invalid { errors, .. } if !errors.is_empty() => errors
            .iter()
            .map(|error| Problem {
                file: file.clone(),
                field: Some(error.field.clone()),
                message: error.message.clone(),
            })
            .collect(),
        other => vec![Problem::whole(file, other.to_string())],
    }
}
