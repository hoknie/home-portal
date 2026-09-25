use std::fmt;
use std::io;
use std::path::PathBuf;

use portal_feature::FieldError;

#[derive(Debug)]
pub enum ConfigError {
    Missing {
        path: PathBuf,
    },
    Unreadable {
        path: PathBuf,
        source: io::Error,
    },
    Syntax {
        path: PathBuf,
        message: String,
    },
    Include {
        path: PathBuf,
        message: String,
    },
    Merge {
        message: String,
    },
    Permissions {
        path: PathBuf,
        mode: u32,
    },
    Invalid {
        path: PathBuf,
        errors: Vec<FieldError>,
    },
}

impl ConfigError {
    pub const EXPECTED_MODE: u32 = 0o600;

    pub fn message(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing { path } => write!(
                formatter,
                "configuration file {} does not exist; create it (see home-portal.example.toml) or set HOME_PORTAL_CONFIG",
                path.display()
            ),
            ConfigError::Unreadable { path, source } => {
                write!(
                    formatter,
                    "cannot read configuration file {}: {source}",
                    path.display()
                )
            }
            ConfigError::Syntax { path, message } => {
                write!(
                    formatter,
                    "configuration file {} is not valid TOML: {message}",
                    path.display()
                )
            }
            ConfigError::Include { path, message } => {
                write!(formatter, "include {}: {message}", path.display())
            }
            ConfigError::Merge { message } => {
                write!(formatter, "the configuration files disagree: {message}")
            }
            ConfigError::Permissions { path, mode } => write!(
                formatter,
                "{} holds secrets and must be readable by its owner only: its mode is {mode:04o}, expected {:04o}",
                path.display(),
                ConfigError::EXPECTED_MODE
            ),
            ConfigError::Invalid { path, errors } => {
                write!(
                    formatter,
                    "configuration file {} is invalid:",
                    path.display()
                )?;
                for error in errors {
                    write!(formatter, "\n  {}: {}", error.field, error.message)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Unreadable { source, .. } => Some(source),
            _ => None,
        }
    }
}
