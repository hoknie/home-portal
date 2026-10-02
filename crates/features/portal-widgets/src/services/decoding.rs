use std::time::Duration;

use portal_feature::FieldError;
use serde_json::Value;

use super::block_checks::check_blocks;
use crate::types::{
    Block, CustomWidget, GroupBlock, RawCustom, RawSource, RowAlign, WidgetPorts, WidgetSource,
};
use portal_widget::Align;

pub fn decode_custom(
    settings: &Value,
    ports: &WidgetPorts,
) -> Result<CustomWidget, Vec<FieldError>> {
    let mut errors = Vec::new();
    let Some(table) = settings.as_object() else {
        return Err(vec![FieldError::new("", "must be a table")]);
    };
    let mut shell = table.clone();
    let raw_blocks = shell.remove(BLOCKS).unwrap_or(Value::Array(Vec::new()));
    let raw: RawCustom = match serde_json::from_value(Value::Object(shell)) {
        Ok(raw) => raw,
        Err(problem) => return Err(vec![FieldError::new("", problem.to_string())]),
    };
    let blocks = decode_blocks(&raw_blocks, BLOCKS, 0, &mut errors);
    let refresh = match raw.refresh_seconds {
        None => CustomWidget::DEFAULT_REFRESH,
        Some(seconds)
            if (CustomWidget::SHORTEST_REFRESH..=CustomWidget::LONGEST_REFRESH)
                .contains(&seconds) =>
        {
            seconds.unsigned_abs()
        }
        Some(_) => {
            errors.push(FieldError::new(
                "refresh_seconds",
                format!(
                    "must be from {} to {} seconds",
                    CustomWidget::SHORTEST_REFRESH,
                    CustomWidget::LONGEST_REFRESH
                ),
            ));
            CustomWidget::DEFAULT_REFRESH
        }
    };
    let source = raw
        .source
        .as_ref()
        .and_then(|source| decode_source(source, ports, &mut errors));
    if blocks.is_empty() && errors.is_empty() {
        errors.push(FieldError::new(BLOCKS, "must hold at least one block"));
    }
    let count = Block::count(&blocks);
    if count > CustomWidget::MOST_BLOCKS {
        errors.push(FieldError::new(
            BLOCKS,
            format!(
                "holds {count} blocks with the nested ones, more than the {} a widget may hold",
                CustomWidget::MOST_BLOCKS
            ),
        ));
    }
    let actions = check_blocks(&blocks, BLOCKS, ports, &mut errors);
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(CustomWidget {
        source,
        refresh: Duration::from_secs(refresh),
        blocks,
        actions,
    })
}

pub const BLOCKS: &str = "blocks";
pub const KIND: &str = "kind";
pub const GROUPS: [&str; 2] = ["row", "column"];

fn decode_blocks(
    value: &Value,
    path: &str,
    depth: usize,
    errors: &mut Vec<FieldError>,
) -> Vec<Block> {
    let Some(list) = value.as_array() else {
        errors.push(FieldError::new(path, "must be a list of blocks"));
        return Vec::new();
    };
    list.iter()
        .enumerate()
        .filter_map(|(index, raw)| {
            let here = format!("{path}[{index}]");
            let kind = raw.get(KIND).and_then(Value::as_str).unwrap_or_default();
            if GROUPS.contains(&kind) {
                return decode_group(raw, kind, &here, depth, errors);
            }
            match serde_json::from_value::<Block>(raw.clone()) {
                Ok(block) => Some(block),
                Err(problem) => {
                    errors.push(FieldError::new(here, problem.to_string()));
                    None
                }
            }
        })
        .collect()
}

pub const DEEPEST_GROUP: usize = 3;

fn field_of<T: serde::de::DeserializeOwned>(
    fields: &mut serde_json::Map<String, Value>,
    key: &str,
    here: &str,
    allowed: &str,
    errors: &mut Vec<FieldError>,
) -> Option<T> {
    let raw = fields.remove(key)?;
    match serde_json::from_value(raw) {
        Ok(value) => Some(value),
        Err(_) => {
            errors.push(FieldError::new(
                format!("{here}.{key}"),
                format!("must be one of {allowed}"),
            ));
            None
        }
    }
}

fn decode_group(
    raw: &Value,
    kind: &str,
    here: &str,
    depth: usize,
    errors: &mut Vec<FieldError>,
) -> Option<Block> {
    let row = kind == "row";
    let mut fields = raw.as_object().cloned().unwrap_or_default();
    fields.remove(KIND);
    let children = fields.remove(BLOCKS).unwrap_or(Value::Array(Vec::new()));
    let gap = field_of(&mut fields, "gap", here, "small, normal", errors).unwrap_or_default();
    let valign: Option<Align> = if row {
        None
    } else {
        field_of(&mut fields, "valign", here, "start, center, end", errors)
    };
    let (align, row_align) = if row {
        let align = field_of(&mut fields, "align", here, RowAlign::NAMES, errors);
        (None, align.unwrap_or_default())
    } else {
        let align: Option<Align> = field_of(
            &mut fields,
            "align",
            here,
            Align::NAMES.trim_end_matches([',', ' ']),
            errors,
        );
        (align, RowAlign::default())
    };
    let widths: Option<Vec<u8>> = if row {
        field_of(
            &mut fields,
            "widths",
            here,
            "whole numbers from 1 to 12",
            errors,
        )
    } else {
        None
    };
    if let Some(unknown) = fields.keys().next() {
        let takes = if row {
            "blocks, gap, align and widths"
        } else {
            "blocks, gap, align and valign"
        };
        errors.push(FieldError::new(
            format!("{here}.{unknown}"),
            format!("is not a field of a {kind}, which takes {takes}"),
        ));
    }
    if depth >= DEEPEST_GROUP {
        errors.push(FieldError::new(
            here,
            format!("groups nest at most {DEEPEST_GROUP} deep: this {kind} is inside {depth}"),
        ));
        return None;
    }
    let path = format!("{here}.{BLOCKS}");
    let blocks = decode_blocks(&children, &path, depth + 1, errors);
    let (fewest, most) = if row { (2, 4) } else { (1, 12) };
    if !(fewest..=most).contains(&blocks.len()) {
        errors.push(FieldError::new(
            &path,
            format!("a {kind} holds from {fewest} to {most} blocks"),
        ));
    }
    if let Some(widths) = &widths
        && (widths.len() != blocks.len() || widths.iter().any(|part| !(1..=12).contains(part)))
    {
        errors.push(FieldError::new(
            format!("{here}.widths"),
            format!(
                "must give one whole number from 1 to 12 for each of the row's {} blocks",
                blocks.len()
            ),
        ));
    }
    let group = GroupBlock {
        blocks,
        gap,
        align,
        valign,
        row_align,
        widths,
    };
    Some(if row {
        Block::Row(group)
    } else {
        Block::Column(group)
    })
}

fn decode_source(
    source: &RawSource,
    ports: &WidgetPorts,
    errors: &mut Vec<FieldError>,
) -> Option<WidgetSource> {
    match (&source.workflow, &source.script) {
        (Some(workflow), None) => {
            if source.script.is_none()
                && (!source.args.is_empty() || source.timeout_seconds.is_some())
            {
                errors.push(FieldError::new(
                    "source",
                    "args and timeout_seconds belong to a script source",
                ));
            }
            let given = source
                .inputs
                .iter()
                .map(|(name, value)| (name.clone(), Some(value.clone())))
                .collect();
            let found = ports.references.call_problems(workflow, &given);
            if !found.is_empty() {
                errors.extend(found.into_iter().map(|error| error.prefixed("source.")));
                return None;
            }
            Some(WidgetSource::Workflow {
                id: workflow.clone(),
                inputs: source.inputs.clone().into_iter().collect(),
            })
        }
        (None, Some(script)) => {
            if !source.inputs.is_empty() {
                errors.push(FieldError::new(
                    "source.inputs",
                    "belongs to a workflow source",
                ));
            }
            if let Some(problem) = ports.references.script_problem(script) {
                errors.push(FieldError::new("source.script", problem));
            }
            let seconds = source
                .timeout_seconds
                .unwrap_or(i64::try_from(CustomWidget::DEFAULT_SCRIPT_TIMEOUT).unwrap_or(30));
            if !(1..=CustomWidget::LONGEST_SCRIPT).contains(&seconds) {
                errors.push(FieldError::new(
                    "source.timeout_seconds",
                    format!("must be from 1 to {} seconds", CustomWidget::LONGEST_SCRIPT),
                ));
            }
            Some(WidgetSource::Script {
                script: script.clone(),
                args: source.args.clone(),
                timeout: Duration::from_secs(seconds.unsigned_abs().max(1)),
            })
        }
        _ => {
            errors.push(FieldError::new(
                "source",
                "needs exactly one of workflow and script",
            ));
            None
        }
    }
}
