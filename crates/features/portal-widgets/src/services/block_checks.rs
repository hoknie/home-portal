use portal_feature::FieldError;
use portal_model::ServiceId;
use portal_widget::Align;

use super::widget_templates::widget_template_problem;
use crate::types::{Block, RawAction, ToneFields, WidgetAction, WidgetPorts, WidgetTarget};

pub const LONGEST_LIST: i64 = 50;
pub const MOST_COLUMNS: usize = 6;
pub const MOST_PAIRS: usize = 12;

pub fn check_blocks(
    blocks: &[Block],
    path: &str,
    ports: &WidgetPorts,
    errors: &mut Vec<FieldError>,
) -> Vec<WidgetAction> {
    let mut actions = Vec::new();
    walk(blocks, path, ports, errors, &mut actions);
    actions
}

fn walk(
    blocks: &[Block],
    path: &str,
    ports: &WidgetPorts,
    errors: &mut Vec<FieldError>,
    actions: &mut Vec<WidgetAction>,
) {
    for (index, block) in blocks.iter().enumerate() {
        let here = format!("{path}[{index}]");
        for (field, value) in [("align", block.align()), ("valign", block.valign())] {
            if value == Some(Align::Unknown) {
                errors.push(FieldError::new(
                    format!("{here}.{field}"),
                    "must be one of start, center, end",
                ));
            }
        }
        let mut template = |field: &str, text: &str, inside_list: bool| {
            if let Some(problem) =
                widget_template_problem(ports.templates.as_ref(), text, inside_list)
            {
                errors.push(FieldError::new(format!("{here}.{field}"), problem));
            }
        };
        match block {
            Block::Stat(stat) => {
                template("label", &stat.label, false);
                template("value", &stat.value, false);
                optional(&mut template, "unit", &stat.unit, false);
                optional(&mut template, "caption", &stat.caption, false);
            }
            Block::Text(text) => template("text", &text.text, false),
            Block::Markdown(markdown) => template("text", &markdown.text, false),
            Block::Badge(badge) => template("text", &badge.text, false),
            Block::List(list) => {
                template("items", &list.items, false);
                template("text", &list.text, true);
                optional(&mut template, "secondary", &list.secondary, true);
                optional(&mut template, "empty", &list.empty, false);
            }
            Block::Table(table) => {
                template("items", &table.items, false);
                optional(&mut template, "empty", &table.empty, false);
                for (column, entry) in table.columns.iter().enumerate() {
                    template(&format!("columns[{column}].header"), &entry.header, false);
                    template(&format!("columns[{column}].value"), &entry.value, true);
                }
            }
            Block::KeyValues(pairs) => {
                for (number, pair) in pairs.pairs.iter().enumerate() {
                    template(&format!("pairs[{number}].key"), &pair.key, false);
                    template(&format!("pairs[{number}].value"), &pair.value, false);
                }
            }
            Block::Progress(progress) => {
                optional(&mut template, "label", &progress.label, false);
                template("value", &progress.value, false);
                optional(&mut template, "caption", &progress.caption, false);
                if let Some(maximum) = &progress.maximum {
                    match maximum {
                        serde_json::Value::Number(_) => {}
                        serde_json::Value::String(text) => template("maximum", text, false),
                        _ => errors.push(FieldError::new(
                            format!("{here}.maximum"),
                            "must be a number or a template",
                        )),
                    }
                }
            }
            Block::Button(button) => {
                template("label", &button.label, false);
                optional(&mut template, "confirm", &button.confirm, false);
                if let Some(action) = check_action(
                    &button.action,
                    &format!("{here}.action"),
                    ports,
                    actions,
                    errors,
                ) {
                    actions.push(action);
                }
            }
            Block::Divider(_) | Block::Row(_) | Block::Column(_) => {}
        }
        check_limits(block, &here, errors);
        if let Some(fields) = block.tone_fields()
            && let Err(problem) = fields.rule()
        {
            errors.push(FieldError::new(format!("{here}.tone"), problem));
        }
        if !block.children().is_empty() {
            walk(
                block.children(),
                &format!("{here}.blocks"),
                ports,
                errors,
                actions,
            );
        }
    }
}

fn optional(
    template: &mut impl FnMut(&str, &str, bool),
    field: &str,
    text: &Option<String>,
    inside_list: bool,
) {
    if let Some(text) = text {
        template(field, text, inside_list);
    }
}

fn check_limits(block: &Block, here: &str, errors: &mut Vec<FieldError>) {
    let limit = match block {
        Block::List(list) => list.limit,
        Block::Table(table) => table.limit,
        _ => None,
    };
    if let Some(limit) = limit
        && !(1..=LONGEST_LIST).contains(&limit)
    {
        errors.push(FieldError::new(
            format!("{here}.limit"),
            format!("must be from 1 to {LONGEST_LIST}"),
        ));
    }
    match block {
        Block::Table(table) if table.columns.is_empty() || table.columns.len() > MOST_COLUMNS => {
            errors.push(FieldError::new(
                format!("{here}.columns"),
                format!("a table has from 1 to {MOST_COLUMNS} columns"),
            ));
        }
        Block::KeyValues(pairs) if pairs.pairs.is_empty() || pairs.pairs.len() > MOST_PAIRS => {
            errors.push(FieldError::new(
                format!("{here}.pairs"),
                format!("holds from 1 to {MOST_PAIRS} pairs"),
            ));
        }
        _ => {}
    }
}

pub fn target_name(target: &WidgetTarget) -> &'static str {
    match target {
        WidgetTarget::Automation { .. } => "automation",
        WidgetTarget::Workflow { .. } => "workflow",
        WidgetTarget::Refresh => "refresh",
        WidgetTarget::Link(_) => "link",
    }
}

fn check_action(
    raw: &RawAction,
    here: &str,
    ports: &WidgetPorts,
    taken: &[WidgetAction],
    errors: &mut Vec<FieldError>,
) -> Option<WidgetAction> {
    let chosen = [
        raw.automation.is_some(),
        raw.workflow.is_some(),
        raw.refresh == Some(true),
        raw.link.is_some(),
    ];
    if chosen.iter().filter(|chosen| **chosen).count() != 1 {
        errors.push(FieldError::new(
            here,
            "needs exactly one of automation, workflow, refresh = true and link",
        ));
        return None;
    }
    let target = if let Some(id) = &raw.automation {
        match ports.references.event_fields(id) {
            None => errors.push(FieldError::new(
                format!("{here}.automation"),
                format!("names {id:?}, which is not an automation"),
            )),
            Some((event, fields)) => {
                for (name, text) in &raw.fields {
                    if !fields.iter().any(|field| field == name) {
                        errors.push(FieldError::new(
                            format!("{here}.fields.{name}"),
                            format!("is not a field of the event {event}"),
                        ));
                    }
                    if let Some(problem) =
                        widget_template_problem(ports.templates.as_ref(), text, false)
                    {
                        errors.push(FieldError::new(format!("{here}.fields.{name}"), problem));
                    }
                }
            }
        }
        WidgetTarget::Automation {
            id: id.clone(),
            fields: raw.fields.clone(),
        }
    } else if let Some(id) = &raw.workflow {
        let given = raw
            .inputs
            .iter()
            .map(|(name, value)| (name.clone(), (!value.is_string()).then(|| value.clone())))
            .collect();
        errors.extend(
            ports
                .references
                .call_problems(id, &given)
                .into_iter()
                .map(|error| error.prefixed(&format!("{here}."))),
        );
        for (name, value) in &raw.inputs {
            if let Some(problem) = value
                .as_str()
                .and_then(|text| widget_template_problem(ports.templates.as_ref(), text, false))
            {
                errors.push(FieldError::new(format!("{here}.inputs.{name}"), problem));
            }
        }
        WidgetTarget::Workflow {
            id: id.clone(),
            inputs: raw.inputs.clone(),
        }
    } else if let Some(link) = &raw.link {
        if let Some(problem) = widget_template_problem(ports.templates.as_ref(), link, false) {
            errors.push(FieldError::new(format!("{here}.link"), problem));
        }
        WidgetTarget::Link(link.clone())
    } else {
        WidgetTarget::Refresh
    };
    let id = match &raw.id {
        Some(id) => {
            if let Err(problem) = ServiceId::parse(id) {
                errors.push(FieldError::new(format!("{here}.id"), problem.to_string()));
            }
            if taken.iter().any(|action| &action.id == id) {
                errors.push(FieldError::new(
                    format!("{here}.id"),
                    "is used by another action of this widget",
                ));
            }
            id.clone()
        }
        None => derived_id(target_name(&target), taken),
    };
    Some(WidgetAction { id, target })
}

fn derived_id(base: &str, taken: &[WidgetAction]) -> String {
    let used = |candidate: &str| taken.iter().any(|action| action.id == candidate);
    if !used(base) {
        return base.to_string();
    }
    (2..)
        .map(|number| format!("{base}-{number}"))
        .find(|candidate| !used(candidate))
        .unwrap_or_else(|| base.to_string())
}

pub fn tone_fields_of(block: &Block) -> ToneFields {
    block.tone_fields().unwrap_or_default()
}
