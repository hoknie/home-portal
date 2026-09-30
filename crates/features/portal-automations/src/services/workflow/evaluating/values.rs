use serde_json::Value;

use super::filters::apply_chain;
use super::frame::Frame;
use super::names_in::placeholder_at;
use super::rendered::record;
use crate::helpers::{CLOSE, OPEN};
use crate::types::{NumberSetting, Placeholder, Workflow};

pub fn oversized(name: &str, value: &Value) -> Option<String> {
    let size = serde_json::to_vec(value).map_or(0, |bytes| bytes.len());
    (size > Workflow::LARGEST_VALUE).then(|| {
        format!(
            "{name} holds {} KiB, more than the {} KiB a value may hold",
            size.div_ceil(1024),
            Workflow::LARGEST_VALUE / 1024
        )
    })
}

pub fn text_of(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

pub fn render_text(template: &str, frame: &Frame) -> Result<String, String> {
    let rendered = text_without_record(template, frame)?;
    record(frame, template, &Value::String(rendered.clone()));
    Ok(rendered)
}

fn text_without_record(template: &str, frame: &Frame) -> Result<String, String> {
    let mut rendered = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find(OPEN) {
        let after = &rest[start + OPEN.len()..];
        match placeholder_at(after) {
            Some(placeholder) => {
                rendered.push_str(&rest[..start]);
                rendered.push_str(&text_of(&evaluate(&placeholder, frame)?));
                rest = &after[placeholder.inner_length + CLOSE.len()..];
            }
            None => {
                rendered.push_str(&rest[..start + 1]);
                rest = &rest[start + 1..];
            }
        }
        if rendered.len() > Workflow::LARGEST_VALUE {
            return Err(too_large());
        }
    }
    rendered.push_str(rest);
    if rendered.len() > Workflow::LARGEST_VALUE {
        return Err(too_large());
    }
    Ok(rendered)
}

pub fn render_value(template: &str, frame: &Frame) -> Result<Value, String> {
    let trimmed = template.trim();
    if let Some(placeholder) = trimmed
        .strip_prefix(OPEN)
        .filter(|inner| inner.ends_with(CLOSE))
        .and_then(placeholder_at)
        .filter(|placeholder| trimmed.len() == placeholder.inner_length + OPEN.len() + CLOSE.len())
    {
        let value = evaluate(&placeholder, frame)?;
        if value.to_string().len() > Workflow::LARGEST_VALUE {
            return Err(too_large());
        }
        record(frame, template, &value);
        return Ok(value);
    }
    render_text(template, frame).map(Value::String)
}

pub fn render_json(template: &str, frame: &Frame) -> Result<Value, String> {
    match render_value(template, frame)? {
        Value::String(text) => {
            serde_json::from_str(&text).map_err(|error| format!("does not render to JSON: {error}"))
        }
        other => Ok(other),
    }
}

fn evaluate(placeholder: &Placeholder<'_>, frame: &Frame) -> Result<Value, String> {
    let value = frame.lookup(placeholder.name)?;
    match &placeholder.filters {
        Ok(filters) => apply_chain(value, filters, &|name| {
            let found = frame.lookup(name)?;
            record(frame, &format!("{OPEN}{name}{CLOSE}"), &found);
            Ok(found)
        }),
        Err(message) => Err(format!("{{{{{}}}}} {message}", placeholder.name)),
    }
}

fn too_large() -> String {
    format!("renders more than {} KiB", Workflow::LARGEST_VALUE / 1024)
}

pub fn render_keys<'a>(
    pairs: &'a [(String, String)],
    frame: &Frame,
) -> Result<Vec<(String, &'a str)>, String> {
    let mut rendered: Vec<(String, &'a str)> = Vec::with_capacity(pairs.len());
    for (index, (key, value)) in pairs.iter().enumerate() {
        let name = if key.contains(OPEN) {
            render_text(key, frame)?
        } else {
            key.clone()
        };
        if name.trim().is_empty() {
            return Err(format!("the key {key} renders empty"));
        }
        if let Some(earlier) = pairs[..index]
            .iter()
            .zip(&rendered)
            .find(|(_, (other, _))| *other == name)
        {
            return Err(format!(
                "the keys {} and {key} both render to \"{name}\"",
                earlier.0.0
            ));
        }
        rendered.push((name, value.as_str()));
    }
    Ok(rendered)
}

pub fn render_number(
    setting: &NumberSetting,
    (minimum, maximum): (u64, u64),
    field: &str,
    frame: &Frame,
) -> Result<u64, String> {
    let template = match setting {
        NumberSetting::Fixed(number) => return Ok(*number),
        NumberSetting::Template(template) => template,
    };
    let value = render_value(template, frame)?;
    let number = match &value {
        Value::Number(number) => number.as_f64(),
        other => text_of(other).trim().parse::<f64>().ok(),
    }
    .filter(|number| number.fract() == 0.0 && *number >= 0.0);
    match number {
        Some(number) if (minimum as f64..=maximum as f64).contains(&number) => Ok(number as u64),
        _ => Err(format!(
            "{field} is {} ({template}), it must be a whole number from {minimum} to {maximum}",
            text_of(&value)
        )),
    }
}
