use portal_feature::FieldError;
use portal_model::{Environment, Environments, Publication};
use url::Url;

use crate::types::ServiceEntry;

pub const NOT_SHOWN: &str = "names an environment this service is not shown in";
pub const UNKNOWN_ENVIRONMENT: &str = "names no configured environment";
pub const GIVEN_BY_PROXY: &str = "is given by proxy.host, because the service is published there";
pub const MAIN_NOT_HTTP: &str =
    "cannot be published: the proxy forwards to the main address, which is not http or https";
pub const RETIRED_UPSTREAM: &str = "is no longer read; the proxy forwards to the main address";

pub fn check_publication(entry: &ServiceEntry, known: &Environments) -> Vec<FieldError> {
    let Some(publication) = &entry.proxy else {
        return Vec::new();
    };
    let mut errors = Vec::new();
    if let Some(message) = Publication::host_problem(&publication.host) {
        errors.push(FieldError::new("proxy.host", message));
    }
    if !speaks_http(main_address(entry)) {
        errors.push(FieldError::new("proxy.host", MAIN_NOT_HTTP));
    }
    for (index, name) in publication.environments.iter().enumerate() {
        let field = format!("proxy.environments[{index}]");
        if !Environment::parse(name).is_ok_and(|environment| known.knows(&environment)) {
            errors.push(FieldError::new(&field, UNKNOWN_ENVIRONMENT));
        } else if entry
            .environments
            .as_ref()
            .is_some_and(|shown| !shown.contains(name))
        {
            errors.push(FieldError::new(&field, NOT_SHOWN));
        }
        if entry.addresses.contains_key(name) {
            errors.push(FieldError::new(format!("addresses.{name}"), GIVEN_BY_PROXY));
        }
    }
    for (index, name) in publication.auth.iter().enumerate() {
        if !Environment::parse(name).is_ok_and(|environment| known.knows(&environment)) {
            errors.push(FieldError::new(
                format!("proxy.auth[{index}]"),
                UNKNOWN_ENVIRONMENT,
            ));
        }
    }
    for (name, message) in publication.tls.iter().flat_map(|tls| tls.problems()) {
        errors.push(FieldError::new(format!("proxy.tls.{name}"), message));
    }
    errors
}

fn main_address(entry: &ServiceEntry) -> &str {
    entry
        .probe
        .environment
        .as_ref()
        .and_then(|name| entry.addresses.get(name))
        .unwrap_or(&entry.url)
}

fn speaks_http(address: &str) -> bool {
    Url::parse(address).is_ok_and(|url| {
        matches!(url.scheme(), "http" | "https")
            && url.host_str().is_some_and(|host| !host.is_empty())
    })
}
