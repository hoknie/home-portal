use portal_feature::FieldError;
use portal_model::ServiceId;
use portal_widget::{WidgetInstance, WidgetsSection};
use toml_edit::DocumentMut;

use super::layout::{DEFAULT_WIDGETS, SECTION, default_layout};

pub fn check_entry(entry: &WidgetInstance) -> Vec<FieldError> {
    let mut errors = Vec::new();
    if entry.kind.is_empty() {
        errors.push(FieldError::new("type", "must not be empty"));
    }
    if let Some(id) = &entry.id
        && let Err(problem) = ServiceId::parse(id)
    {
        errors.push(FieldError::new("id", problem.to_string()));
    }
    if !entry.settings.is_object() {
        errors.push(FieldError::new("settings", "must be a table"));
    }
    for (key, message) in entry.layout_problems() {
        if key == "width" || key == "height" || key.starts_with("appearance.") {
            errors.push(FieldError::new(key, message));
        }
    }
    errors
}

pub fn library_views(document: &DocumentMut) -> Result<Vec<(WidgetInstance, usize)>, String> {
    if document.get(SECTION).is_none() {
        return Ok(default_layout()
            .widgets
            .into_iter()
            .map(|widget| (widget, 1))
            .collect());
    }
    let placements = WidgetsSection::placements(document)?;
    let library = WidgetsSection::library(document)?;
    let named = WidgetsSection::with_ids(&placements, &library);
    Ok(WidgetsSection::named_definitions(document)?
        .into_iter()
        .map(|entry| {
            let placed = named
                .iter()
                .filter(|placement| {
                    placement.widget.as_ref().or(placement.id.as_ref()) == entry.id.as_ref()
                })
                .count();
            (entry, placed)
        })
        .collect())
}

pub fn library_ids(document: &DocumentMut) -> Vec<String> {
    if document.get(SECTION).is_none() {
        return DEFAULT_WIDGETS
            .iter()
            .map(|kind| kind.to_string())
            .collect();
    }
    WidgetsSection::named_definitions(document)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| entry.id)
        .collect()
}
