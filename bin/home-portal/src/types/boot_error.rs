use std::fmt;
use std::io;
use std::net::SocketAddr;

use portal_config::ConfigError;

#[derive(Debug)]
pub enum BootError {
    Address {
        value: String,
        variable: &'static str,
    },
    Configuration(ConfigError),
    Feature {
        name: &'static str,
        message: String,
    },
    Bind {
        address: SocketAddr,
        source: io::Error,
    },
    Serve {
        address: SocketAddr,
        source: io::Error,
    },
}

impl fmt::Display for BootError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BootError::Address { value, variable } => {
                write!(
                    formatter,
                    "{variable}={value:?} is not a socket address such as 127.0.0.1:8080"
                )
            }
            BootError::Configuration(error) => write!(formatter, "{error}"),
            BootError::Feature { name, message } => {
                write!(formatter, "the {name} feature cannot start: {message}")
            }
            BootError::Bind { address, source } => {
                write!(formatter, "cannot listen on {address}: {source}")
            }
            BootError::Serve { address, source } => {
                write!(formatter, "serving on {address} failed: {source}")
            }
        }
    }
}

impl std::error::Error for BootError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BootError::Configuration(error) => Some(error),
            BootError::Bind { source, .. } | BootError::Serve { source, .. } => Some(source),
            BootError::Address { .. } | BootError::Feature { .. } => None,
        }
    }
}

impl From<ConfigError> for BootError {
    fn from(error: ConfigError) -> BootError {
        BootError::Configuration(error)
    }
}
