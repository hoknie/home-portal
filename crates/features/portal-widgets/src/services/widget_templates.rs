use crate::ports::{NameKind, WidgetTemplates};

const WIDGET_NAMES: &str = "data, data.<path>, widget.id, widget.title and fetched_at, and inside a list item, item.<path> and index";

pub fn widget_template_problem(
    templates: &dyn WidgetTemplates,
    template: &str,
    inside_list: bool,
) -> Option<String> {
    templates.problem(template, &|name| widget_allows(name, inside_list))
}

fn widget_allows(name: &str, inside_list: bool) -> Result<NameKind, String> {
    let mut parts = name.split('.');
    let namespace = parts.next().unwrap_or_default();
    let first = parts.next();
    let fine = match namespace {
        "data" | "fetched_at" => true,
        "widget" => matches!(first, Some("id" | "title")),
        "item" | "index" => inside_list,
        _ => false,
    };
    if fine {
        return Ok(match namespace {
            "index" => NameKind::Number,
            "widget" | "fetched_at" => NameKind::Text,
            _ => NameKind::Any,
        });
    }
    Err(match namespace {
        "item" | "index" => format!(
            "names {{{{{name}}}}}, which exists only in the fields of a list or a table that are read for each item"
        ),
        _ => format!(
            "names {{{{{name}}}}}, which a widget does not know; a widget may use {WIDGET_NAMES}"
        ),
    })
}
