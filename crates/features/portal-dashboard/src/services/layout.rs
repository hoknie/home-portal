use std::collections::HashSet;

use portal_feature::FieldError;
use portal_model::ServiceId;
use portal_widget::{Layout, SectionEntry, WidgetInstance, WidgetsSection};
use toml_edit::DocumentMut;

pub const SECTION: &str = "dashboard";
pub const STATUS_SUMMARY: &str = "status-summary";
pub const SERVICES: &str = "services";
pub const DEFAULT_WIDGETS: [&str; 2] = [STATUS_SUMMARY, SERVICES];

pub fn default_layout() -> Layout {
    let mut summary = WidgetInstance::of(STATUS_SUMMARY);
    summary.id = Some(STATUS_SUMMARY.to_string());
    summary.section = Some(SectionEntry::IMPLICIT.to_string());
    let mut services = WidgetInstance::of(SERVICES);
    services.id = Some(SERVICES.to_string());
    services.section = Some(SectionEntry::IMPLICIT.to_string());
    Layout {
        sections: vec![SectionEntry::implicit()],
        widgets: vec![summary, services],
        explicit_sections: false,
    }
}

pub fn layout(document: &DocumentMut) -> Result<Layout, String> {
    if document.get(SECTION).is_none() {
        return Ok(default_layout());
    }
    Ok(WidgetsSection::layout(document)?.unwrap_or_else(default_layout))
}

pub fn validate_dashboard(document: &DocumentMut) -> Vec<FieldError> {
    let layout = match layout(document) {
        Err(message) => return vec![FieldError::new(SECTION, message)],
        Ok(layout) => layout,
    };
    let placements = WidgetsSection::placements(document).unwrap_or_default();
    let library = WidgetsSection::library(document).unwrap_or_default();
    let mut errors = check_sections(&layout);
    errors.extend(check_library(&library, &placements));
    let known: Vec<&str> = if layout.explicit_sections {
        layout
            .sections
            .iter()
            .map(|section| section.id.as_str())
            .collect()
    } else {
        Vec::new()
    };
    for (index, widget) in placements.iter().enumerate() {
        let field = format!("{SECTION}.widgets[{index}]");
        match &widget.widget {
            Some(name) => errors.extend(check_reference(&field, name, widget, &library)),
            None => {
                if widget.kind.trim().is_empty() {
                    errors.push(FieldError::new(
                        format!("{field}.type"),
                        "must not be empty",
                    ));
                }
                if !widget.settings.is_object() {
                    errors.push(FieldError::new(
                        format!("{field}.settings"),
                        "must be a table",
                    ));
                }
            }
        }
        for (key, message) in widget.layout_problems() {
            errors.push(FieldError::new(format!("{field}.{key}"), message));
        }
        if let Some(section) = &widget.section
            && !known.contains(&section.as_str())
        {
            errors.push(FieldError::new(
                format!("{field}.section"),
                format!("names {section}, and no [[dashboard.sections]] entry has that id"),
            ));
        }
    }
    errors
}

fn check_sections(layout: &Layout) -> Vec<FieldError> {
    if !layout.explicit_sections {
        return Vec::new();
    }
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    for (index, section) in layout.sections.iter().enumerate() {
        let field = format!("{SECTION}.sections[{index}].id");
        if let Err(problem) = ServiceId::parse(&section.id) {
            errors.push(FieldError::new(&field, problem.to_string()));
        }
        if !seen.insert(section.id.as_str()) {
            errors.push(FieldError::new(&field, "is used by another section"));
        }
        for (key, message) in section.appearance.problems() {
            errors.push(FieldError::new(
                format!("{SECTION}.sections[{index}].appearance.{key}"),
                message,
            ));
        }
    }
    errors
}

fn check_reference(
    field: &str,
    name: &str,
    placement: &WidgetInstance,
    library: &[WidgetInstance],
) -> Vec<FieldError> {
    let mut errors = Vec::new();
    if !library
        .iter()
        .any(|entry| entry.id.as_deref() == Some(name))
    {
        errors.push(FieldError::new(
            format!("{field}.widget"),
            format!("names {name}, and no [[dashboard.library]] entry has that id"),
        ));
    }
    let defined = [
        ("type", !placement.kind.is_empty()),
        ("id", placement.id.is_some()),
        ("title", placement.title.is_some()),
        (
            "settings",
            placement
                .settings
                .as_object()
                .is_some_and(|fields| !fields.is_empty()),
        ),
        ("environments", placement.environments.is_some()),
        ("public", placement.public),
        ("appearance", !placement.appearance.is_default()),
    ];
    for (key, present) in defined {
        if present {
            errors.push(FieldError::new(
                format!("{field}.{key}"),
                format!("belongs to the library widget {name}, not to its place on the page"),
            ));
        }
    }
    errors
}

fn check_library(library: &[WidgetInstance], placements: &[WidgetInstance]) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    let inline: Vec<&str> = placements
        .iter()
        .filter(|placement| placement.is_inline())
        .filter_map(|placement| placement.id.as_deref())
        .collect();
    for (index, entry) in library.iter().enumerate() {
        let field = format!("{SECTION}.library[{index}]");
        match &entry.id {
            None => errors.push(FieldError::new(
                format!("{field}.id"),
                "a library widget needs an id",
            )),
            Some(id) => {
                if let Err(problem) = ServiceId::parse(id) {
                    errors.push(FieldError::new(format!("{field}.id"), problem.to_string()));
                }
                if !seen.insert(id.as_str()) || inline.contains(&id.as_str()) {
                    errors.push(FieldError::new(
                        format!("{field}.id"),
                        "is used by another widget",
                    ));
                }
            }
        }
        if entry.kind.trim().is_empty() {
            errors.push(FieldError::new(
                format!("{field}.type"),
                "must not be empty",
            ));
        }
        if !entry.settings.is_object() {
            errors.push(FieldError::new(
                format!("{field}.settings"),
                "must be a table",
            ));
        }
        for (key, message) in entry.appearance.problems() {
            errors.push(FieldError::new(
                format!("{field}.appearance.{key}"),
                message,
            ));
        }
        let placed = [
            ("widget", entry.widget.is_some()),
            ("section", entry.section.is_some()),
            ("column", entry.column.is_some()),
            ("row", entry.row.is_some()),
            ("size", entry.size.is_some()),
        ];
        for (key, present) in placed {
            if present {
                errors.push(FieldError::new(
                    format!("{field}.{key}"),
                    "belongs to a place on the page, in [[dashboard.widgets]]",
                ));
            }
        }
        for (key, message) in entry.layout_problems() {
            if key == "width" || key == "height" {
                errors.push(FieldError::new(format!("{field}.{key}"), message));
            }
        }
    }
    errors
}
