use std::collections::HashSet;

use portal_feature::FieldError;
use portal_model::ServiceId;
use portal_widget::WidgetSize;

use crate::types::EditedLayout;

pub fn check_edited(edited: &EditedLayout) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let mut sections = HashSet::new();
    if edited.sections.is_empty() {
        errors.push(FieldError::new(
            "sections",
            "must hold at least one section",
        ));
    }
    for (index, section) in edited.sections.iter().enumerate() {
        let field = format!("sections[{index}].id");
        if let Err(problem) = ServiceId::parse(&section.id) {
            errors.push(FieldError::new(&field, problem.to_string()));
        }
        if !sections.insert(section.id.as_str()) {
            errors.push(FieldError::new(&field, "is used by another section"));
        }
    }
    let mut ids = HashSet::new();
    for (index, widget) in edited.widgets.iter().enumerate() {
        let prefix = format!("widgets[{index}]");
        let instance = &widget.instance;
        if instance.kind.is_empty() {
            errors.push(FieldError::new(
                format!("{prefix}.type"),
                "must not be empty",
            ));
        }
        if let Some(id) = &instance.id {
            if let Err(problem) = ServiceId::parse(id) {
                errors.push(FieldError::new(format!("{prefix}.id"), problem.to_string()));
            }
            if !ids.insert(id.as_str()) {
                errors.push(FieldError::new(
                    format!("{prefix}.id"),
                    "is used by another widget",
                ));
            }
        }
        if instance.size == WidgetSize::Unknown {
            errors.push(FieldError::new(
                format!("{prefix}.size"),
                format!("must be one of {}", WidgetSize::NAMES),
            ));
        }
        if !instance.settings.is_object() {
            errors.push(FieldError::new(
                format!("{prefix}.settings"),
                "must be a table",
            ));
        }
        match &instance.section {
            Some(section) if sections.contains(section.as_str()) => {}
            Some(_) | None => errors.push(FieldError::new(
                format!("{prefix}.section"),
                "must name one of the sections",
            )),
        }
    }
    errors
}

pub fn renamed_for_the_editor(error: FieldError, edited: &EditedLayout) -> FieldError {
    let field = error.field.clone();
    if let Some(rest) = field.strip_prefix("dashboard.widgets[") {
        return FieldError::new(format!("widgets[{rest}"), error.message);
    }
    if let Some(rest) = field.strip_prefix("dashboard.sections[") {
        return FieldError::new(format!("sections[{rest}"), error.message);
    }
    if let Some(rest) = field.strip_prefix("dashboard.widgets.") {
        let index = edited.widgets.iter().position(|widget| {
            widget
                .instance
                .id
                .as_deref()
                .is_some_and(|id| rest.starts_with(&format!("{id}.")))
        });
        if let Some(index) = index {
            let id = edited.widgets[index]
                .instance
                .id
                .as_deref()
                .unwrap_or_default();
            let tail = &rest[id.len()..];
            return FieldError::new(format!("widgets[{index}]{tail}"), error.message);
        }
    }
    error
}
