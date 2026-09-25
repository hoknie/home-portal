use portal_feature::FieldError;
use url::Url;

use crate::helpers::tcp_port;
use crate::types::{ProbeKind, ProbeSettings, ServiceEntry};

pub fn address_problem(address: &str, kind: ProbeKind) -> Option<String> {
    let parsed = match Url::parse(address) {
        Err(_) if kind.speaks_http() => {
            return Some("must be an absolute http or https URL".to_string());
        }
        Err(_) => return Some("must be an absolute address with a host".to_string()),
        Ok(parsed) => parsed,
    };
    if kind.speaks_http() && !matches!(parsed.scheme(), "http" | "https") {
        return Some("must use http or https".to_string());
    }
    if parsed.host_str().is_none_or(str::is_empty) {
        return Some("must name a host".to_string());
    }
    None
}

pub fn check_probe(entry: &ServiceEntry) -> Vec<FieldError> {
    let probe = &entry.probe;
    let mut errors = Vec::new();
    if !probe.path.starts_with('/') {
        errors.push(FieldError::new("probe.path", "must start with /"));
    }
    if probe.port == Some(0) {
        errors.push(FieldError::new("probe.port", "must be between 1 and 65535"));
    } else if probe.kind == ProbeKind::Tcp
        && address_problem(&entry.url, probe.kind).is_none()
        && let Err(message) = tcp_port(&entry.url, probe.port)
    {
        errors.push(FieldError::new("probe.port", message));
    }
    if !ProbeSettings::EVERY_SECONDS.contains(&probe.every_seconds) {
        errors.push(FieldError::new(
            "probe.every_seconds",
            "must be between 5 and 3600",
        ));
    }
    if !ProbeSettings::TIMEOUT_SECONDS.contains(&probe.timeout_seconds) {
        errors.push(FieldError::new(
            "probe.timeout_seconds",
            "must be between 1 and 60",
        ));
    } else if probe.timeout_seconds >= probe.every_seconds {
        errors.push(FieldError::new(
            "probe.timeout_seconds",
            "must be less than every_seconds",
        ));
    }
    if !ProbeSettings::DEGRADED_AFTER_MILLISECONDS.contains(&probe.degraded_after_milliseconds) {
        errors.push(FieldError::new(
            "probe.degraded_after_milliseconds",
            "must be between 1 and 60000",
        ));
    }
    errors
}
