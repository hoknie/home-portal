use std::collections::HashSet;

use portal_config::deserialize_section;
use portal_feature::FieldError;
use portal_model::{Environment, Environments, RawEnvironmentsSection, ServiceId};
use portal_widget::WidgetsSection;
use toml_edit::DocumentMut;

use super::details_validation::{check_links, check_notes, check_widgets};
use super::probe_validation::{address_problem, check_probe};
use super::publishing::check_publication;
use crate::types::{Known, ServiceEntry, ServicesSection};

pub fn validate_services(document: &DocumentMut) -> Vec<FieldError> {
    let section = match ServicesSection::read(document) {
        Ok(section) => section,
        Err(message) => return vec![FieldError::new("services", message)],
    };
    let known = known_of(document);
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    for (index, entry) in section.services.iter().enumerate() {
        let prefix = format!("services[{index}].");
        errors.extend(
            check_entry(entry, &known)
                .into_iter()
                .map(|error| error.prefixed(&prefix)),
        );
        if !seen.insert(entry.id.as_str()) {
            errors.push(FieldError::new(
                format!("{prefix}id"),
                "is used by another service",
            ));
        }
    }
    errors
}

pub fn known_of(document: &DocumentMut) -> Known {
    Known {
        environments: known_environments(document),
        widgets: WidgetsSection::read(document)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|widget| widget.id)
            .collect(),
    }
}

pub fn known_environments(document: &DocumentMut) -> Environments {
    deserialize_section::<RawEnvironmentsSection>(document)
        .ok()
        .map(|section| {
            Environments::new(
                section
                    .environments
                    .keys()
                    .filter_map(|name| Environment::parse(name).ok())
                    .map(|environment| (environment, Vec::new()))
                    .collect(),
            )
        })
        .unwrap_or_default()
}

pub fn check_entry(entry: &ServiceEntry, known: &Known) -> Vec<FieldError> {
    let mut errors = Vec::new();
    if let Err(error) = ServiceId::parse(&entry.id) {
        errors.push(FieldError::new("id", error.to_string()));
    }
    let name = entry.name.trim();
    if name.is_empty() || name.chars().count() > ServiceEntry::MAXIMUM_NAME_LENGTH {
        errors.push(FieldError::new(
            "name",
            format!(
                "must be 1 to {} characters",
                ServiceEntry::MAXIMUM_NAME_LENGTH
            ),
        ));
    }
    if let Some(message) = address_problem(&entry.url, entry.probe.kind) {
        errors.push(FieldError::new("url", message));
    }
    if entry.description.as_ref().is_some_and(|description| {
        description.chars().count() > ServiceEntry::MAXIMUM_DESCRIPTION_LENGTH
    }) {
        errors.push(FieldError::new(
            "description",
            format!(
                "must be at most {} characters",
                ServiceEntry::MAXIMUM_DESCRIPTION_LENGTH
            ),
        ));
    }
    errors.extend(check_addresses(entry, &known.environments));
    errors.extend(check_environments(entry, &known.environments));
    errors.extend(check_links(entry));
    errors.extend(check_notes(entry));
    errors.extend(check_widgets(entry, &known.widgets));
    errors.extend(check_probe(entry));
    errors.extend(check_probe_environment(entry, &known.environments));
    errors.extend(check_publication(entry, &known.environments));
    errors
}

fn check_addresses(entry: &ServiceEntry, known: &Environments) -> Vec<FieldError> {
    let mut errors = Vec::new();
    for (name, address) in &entry.addresses {
        let field = format!("addresses.{name}");
        match Environment::parse(name) {
            Ok(environment) if known.knows(&environment) => {}
            _ => errors.push(FieldError::new(&field, "names no configured environment")),
        }
        if let Some(message) = address_problem(address, entry.probe.kind) {
            errors.push(FieldError::new(&field, message));
        }
    }
    errors
}

fn check_environments(entry: &ServiceEntry, known: &Environments) -> Vec<FieldError> {
    entry
        .environments
        .iter()
        .flatten()
        .enumerate()
        .filter(|(_, name)| {
            !Environment::parse(name).is_ok_and(|environment| known.knows(&environment))
        })
        .map(|(index, _)| {
            FieldError::new(
                format!("environments[{index}]"),
                "names no configured environment",
            )
        })
        .collect()
}

fn check_probe_environment(entry: &ServiceEntry, known: &Environments) -> Vec<FieldError> {
    let Some(name) = &entry.probe.environment else {
        return Vec::new();
    };
    let mut errors = Vec::new();
    if !Environment::parse(name).is_ok_and(|environment| known.knows(&environment)) {
        errors.push(FieldError::new(
            "probe.environment",
            "names no configured environment",
        ));
    } else if !entry.addresses.contains_key(name) {
        errors.push(FieldError::new(
            "probe.environment",
            format!("names {name}, and this service has no address for it"),
        ));
    }
    errors
}
