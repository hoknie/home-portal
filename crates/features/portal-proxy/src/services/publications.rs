use std::collections::HashMap;

use portal_config::deserialize_section;
use portal_feature::FieldError;
use portal_model::Publication;
use toml_edit::DocumentMut;

use super::read_settings;
use crate::types::{ProxySettings, RawServicesView};

pub const TAKEN_HOST: &str = "is published by another service";
pub const PORTAL_HOST: &str = "is the portal's own host, proxy.portal_host";
pub const OUTSIDE_COOKIE_DOMAIN: &str =
    "must be under proxy.cookie_domain for the portal's sign-in to reach it";
pub const NO_COOKIE_DOMAIN: &str =
    "needs proxy.cookie_domain, so that the portal's sign-in reaches this host";

pub fn validate_publications(document: &DocumentMut) -> Vec<FieldError> {
    let Ok(view) = deserialize_section::<RawServicesView>(document) else {
        return Vec::new();
    };
    let settings = read_settings(document).unwrap_or_default();
    let mut errors = Vec::new();
    let mut hosts: HashMap<&str, &str> = HashMap::new();
    for (index, service) in view.services.iter().enumerate() {
        let Some(publication) = &service.proxy else {
            continue;
        };
        let field = format!("services[{index}].proxy.host");
        if hosts
            .insert(publication.host.as_str(), service.id.as_str())
            .is_some()
        {
            errors.push(FieldError::new(&field, TAKEN_HOST));
        }
        if settings.enabled {
            errors.extend(
                against_settings(publication, &settings).map(|(name, message)| {
                    FieldError::new(format!("services[{index}].proxy.{name}"), message)
                }),
            );
        }
    }
    errors
}

pub fn publication_problems(document: &DocumentMut, publication: &Publication) -> Vec<FieldError> {
    let settings = read_settings(document).unwrap_or_default();
    if !settings.enabled {
        return Vec::new();
    }
    against_settings(publication, &settings)
        .map(|(name, message)| FieldError::new(format!("proxy.{name}"), message))
        .into_iter()
        .collect()
}

fn against_settings(
    publication: &Publication,
    settings: &ProxySettings,
) -> Option<(&'static str, &'static str)> {
    if settings.portal_host.as_deref() == Some(publication.host.as_str()) {
        return Some(("host", PORTAL_HOST));
    }
    if publication.auth.is_empty() {
        return None;
    }
    match &settings.cookie_domain {
        Some(domain) if !Publication::is_within(&publication.host, domain) => {
            Some(("auth", OUTSIDE_COOKIE_DOMAIN))
        }
        None => Some(("auth", NO_COOKIE_DOMAIN)),
        Some(_) => None,
    }
}
