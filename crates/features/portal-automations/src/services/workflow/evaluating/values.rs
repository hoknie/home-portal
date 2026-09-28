use serde_json::Value;

use super::filters::apply_chain;
use super::frame::Frame;
use super::names_in::placeholder_at;
use crate::helpers::{CLOSE, OPEN};
use crate::types::{Placeholder, Workflow};

pub fn text_of(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

pub fn render_text(template: &str, frame: &Frame) -> Result<String, String> {
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
        Ok(filters) => apply_chain(value, filters),
        Err(message) => Err(format!("{{{{{}}}}} {message}", placeholder.name)),
    }
}

fn too_large() -> String {
    format!("renders more than {} KiB", Workflow::LARGEST_VALUE / 1024)
}
