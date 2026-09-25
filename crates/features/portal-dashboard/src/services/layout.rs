use std::collections::HashSet;

use portal_feature::FieldError;
use portal_model::ServiceId;
use portal_widget::{Layout, SectionEntry, WidgetInstance, WidgetSize, WidgetsSection};
use toml_edit::DocumentMut;

pub const SECTION: &str = "dashboard";
pub const STATUS_SUMMARY: &str = "status-summary";
pub const SERVICES: &str = "services";

pub fn default_layout() -> Layout {
    let mut summary = WidgetInstance::of(STATUS_SUMMARY);
    summary.section = Some(SectionEntry::IMPLICIT.to_string());
    let mut services = WidgetInstance::of(SERVICES);
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
    let raw = WidgetsSection::read(document).unwrap_or_default();
    let mut errors = check_sections(&layout);
    let known: Vec<&str> = if layout.explicit_sections {
        layout
            .sections
            .iter()
            .map(|section| section.id.as_str())
            .collect()
    } else {
        Vec::new()
    };
    for (index, widget) in raw.iter().enumerate() {
        let field = format!("{SECTION}.widgets[{index}]");
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
        if widget.size == WidgetSize::Unknown {
            errors.push(FieldError::new(
                format!("{field}.size"),
                format!("must be one of {}", WidgetSize::NAMES),
            ));
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
    }
    errors
}
