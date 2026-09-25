use serde::{Deserialize, Serialize};

use super::TlsPolicy;
use crate::types::Environment;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Publication {
    pub host: String,
    #[serde(default)]
    pub upstream: Option<String>,
    #[serde(default = "Publication::default_environments")]
    pub environments: Vec<String>,
    #[serde(default)]
    pub auth: Vec<String>,
    #[serde(default)]
    pub tls: Option<TlsPolicy>,
    #[serde(default = "Publication::verified")]
    pub upstream_verify: bool,
}

impl Publication {
    pub const MAXIMUM_HOST_LENGTH: usize = 253;
    pub const MAXIMUM_LABEL_LENGTH: usize = 63;
    pub const HTTPS_PORT: u16 = 443;
    pub const HTTP_PORT: u16 = 80;

    pub fn new(host: &str) -> Publication {
        Publication {
            host: host.to_string(),
            upstream: None,
            environments: Self::default_environments(),
            auth: Vec::new(),
            tls: None,
            upstream_verify: true,
        }
    }

    pub fn default_environments() -> Vec<String> {
        vec![Environment::INTERNET.to_string()]
    }

    pub fn verified() -> bool {
        true
    }

    pub fn address_on(&self, https_port: u16) -> String {
        Self::origin(&self.host, https_port)
    }

    pub fn origin(host: &str, https_port: u16) -> String {
        if https_port == Self::HTTPS_PORT {
            format!("https://{host}")
        } else {
            format!("https://{host}:{https_port}")
        }
    }

    pub fn published_in(&self, environment: &Environment) -> bool {
        self.environments
            .iter()
            .any(|name| name == environment.as_str())
    }

    pub fn host_problem(host: &str) -> Option<&'static str> {
        if host.is_empty() {
            return Some("must not be empty");
        }
        if host.len() > Self::MAXIMUM_HOST_LENGTH {
            return Some("must be at most 253 characters");
        }
        if host.contains(['/', ':', '@', ' ']) {
            return Some("must be a host name alone, without a scheme, port or path");
        }
        let valid_label = |label: &str| {
            !label.is_empty()
                && label.len() <= Self::MAXIMUM_LABEL_LENGTH
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label.chars().all(|character| {
                    character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
                })
        };
        if !host.split('.').all(valid_label) {
            return Some(
                "must be lower-case labels of letters, digits and hyphens, separated by dots",
            );
        }
        None
    }

    pub fn is_within(host: &str, domain: &str) -> bool {
        host == domain
            || host
                .strip_suffix(domain)
                .is_some_and(|prefix| prefix.ends_with('.'))
    }
}
