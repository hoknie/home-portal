use portal_feature::FieldError;
use toml_edit::{DocumentMut, Item};

use crate::types::PermissionSettings;

pub fn read_settings(document: &DocumentMut) -> PermissionSettings {
    let defaults = PermissionSettings::default();
    let Some(table) = document
        .get(PermissionSettings::TABLE)
        .and_then(Item::as_table_like)
    else {
        return defaults;
    };
    PermissionSettings {
        request_at_start: table
            .get(PermissionSettings::REQUEST_AT_START)
            .and_then(Item::as_bool)
            .unwrap_or(defaults.request_at_start),
        automation: texts(table.get(PermissionSettings::AUTOMATION)).unwrap_or(defaults.automation),
        folders: texts(table.get(PermissionSettings::FOLDERS)).unwrap_or(defaults.folders),
    }
}

pub fn validate_permissions(document: &DocumentMut) -> Vec<FieldError> {
    let Some(item) = document.get(PermissionSettings::TABLE) else {
        return Vec::new();
    };
    let Some(table) = item.as_table_like() else {
        return vec![FieldError::new(
            PermissionSettings::TABLE,
            "must be a table",
        )];
    };
    let mut errors = Vec::new();
    for (key, value) in table.iter() {
        let field = format!("{}.{key}", PermissionSettings::TABLE);
        let problem = match key {
            PermissionSettings::REQUEST_AT_START => {
                value.as_bool().is_none().then_some("must be true or false")
            }
            PermissionSettings::AUTOMATION => application_problem(value),
            PermissionSettings::FOLDERS => folder_problem(value),
            _ => Some("is not a known key; the keys are request_at_start, automation and folders"),
        };
        if let Some(problem) = problem {
            errors.push(FieldError::new(field, problem));
        }
    }
    errors
}

fn texts(item: Option<&Item>) -> Option<Vec<String>> {
    let array = item?.as_array()?;
    array
        .iter()
        .map(|value| value.as_str().map(str::to_string))
        .collect()
}

fn application_problem(value: &Item) -> Option<&'static str> {
    match texts(Some(value)) {
        None => Some("must be a list of application names"),
        Some(names) if names.iter().any(|name| name.trim().is_empty()) => {
            Some("must not hold an empty application name")
        }
        Some(_) => None,
    }
}

fn folder_problem(value: &Item) -> Option<&'static str> {
    match texts(Some(value)) {
        Some(names)
            if names
                .iter()
                .all(|name| PermissionSettings::KNOWN_FOLDERS.contains(&name.as_str())) =>
        {
            None
        }
        _ => Some("must be a list of Documents, Desktop and Downloads"),
    }
}
