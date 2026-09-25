use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use portal_feature::FieldError;
use toml_edit::{DocumentMut, Item};

use crate::types::Storage;

pub const STORAGE_SECTION: &str = "storage";
pub const DIRECTORY_KEY: &str = "directory";

pub fn storage_errors(main: &Path, document: &DocumentMut) -> Vec<FieldError> {
    let mut errors = key_errors(document);
    if errors.is_empty() {
        errors.extend(overlap_errors(main, document));
    }
    errors
}

fn key_errors(document: &DocumentMut) -> Vec<FieldError> {
    let Some(item) = document.get(STORAGE_SECTION) else {
        return Vec::new();
    };
    let Some(table) = item.as_table_like() else {
        return vec![FieldError::new(STORAGE_SECTION, "must be a table")];
    };
    table
        .iter()
        .filter_map(|(key, value)| {
            let field = format!("{STORAGE_SECTION}.{key}");
            let known = key == DIRECTORY_KEY || Storage::ALL.iter().any(|kind| kind.key() == key);
            if !known {
                return Some(FieldError::new(
                    field,
                    format!(
                        "unknown key; expected {DIRECTORY_KEY} or one of {}",
                        Storage::ALL.map(Storage::key).join(", ")
                    ),
                ));
            }
            match value.as_str() {
                Some(text) if !text.trim().is_empty() => None,
                _ => Some(FieldError::new(field, "must be a non-empty path")),
            }
        })
        .collect()
}

fn overlap_errors(main: &Path, document: &DocumentMut) -> Vec<FieldError> {
    let places = storage_places(main, document);
    let scripts = lexical(&places[&Storage::Scripts]);
    places
        .iter()
        .filter(|(kind, _)| **kind != Storage::Scripts)
        .filter(|(_, place)| {
            let place = lexical(place);
            place.starts_with(&scripts) || scripts.starts_with(&place)
        })
        .map(|(kind, _)| {
            FieldError::new(
                format!("{STORAGE_SECTION}.{}", Storage::Scripts.key()),
                format!(
                    "must not overlap the {} place: the portal writes there, and scripts run only from a directory it never writes",
                    kind.key()
                ),
            )
        })
        .collect()
}

fn lexical(path: &Path) -> PathBuf {
    let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let mut clean = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::ParentDir => {
                clean.pop();
            }
            Component::CurDir => {}
            other => clean.push(other),
        }
    }
    clean
}

pub fn storage_places(main: &Path, document: &DocumentMut) -> BTreeMap<Storage, PathBuf> {
    let base = main
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let table = document.get(STORAGE_SECTION).and_then(Item::as_table_like);
    let setting = |key: &str| {
        table
            .and_then(|table| table.get(key))
            .and_then(Item::as_str)
            .filter(|text| !text.trim().is_empty())
            .map(|text| base.join(text))
    };
    let directory = setting(DIRECTORY_KEY).unwrap_or_else(|| base.to_path_buf());
    Storage::ALL
        .into_iter()
        .map(|kind| {
            let place = setting(kind.key()).unwrap_or_else(|| directory.join(kind.default_name()));
            (kind, place)
        })
        .collect()
}
