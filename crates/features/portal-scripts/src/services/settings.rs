use portal_feature::FieldError;
use toml_edit::{DocumentMut, Item};

use crate::types::ScriptsSettings;

pub fn scripts_settings(document: &DocumentMut) -> ScriptsSettings {
    let editing = document
        .get(ScriptsSettings::TABLE)
        .and_then(|table| table.get(ScriptsSettings::EDITING))
        .and_then(Item::as_bool)
        .unwrap_or(false);
    ScriptsSettings { editing }
}

pub fn validate_scripts(document: &DocumentMut) -> Vec<FieldError> {
    let Some(item) = document.get(ScriptsSettings::TABLE) else {
        return Vec::new();
    };
    let Some(table) = item.as_table_like() else {
        return vec![FieldError::new(ScriptsSettings::TABLE, "must be a table")];
    };
    let mut errors = Vec::new();
    for (key, value) in table.iter() {
        let field = format!("{}.{key}", ScriptsSettings::TABLE);
        if key != ScriptsSettings::EDITING {
            errors.push(FieldError::new(
                field,
                format!(
                    "is not a known key; the only key is {}",
                    ScriptsSettings::EDITING
                ),
            ));
        } else if value.as_bool().is_none() {
            errors.push(FieldError::new(field, "must be true or false"));
        }
    }
    errors
}
