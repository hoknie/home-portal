use serde_json::Value;

use super::names_in::valid_name;
use crate::types::FilterCall;

pub fn parse_chain(text: &str) -> Result<Vec<FilterCall>, String> {
    split_outside_quotes(text, '|')
        .into_iter()
        .map(|part| parse_call(part.trim()).ok_or_else(|| unreadable(part.trim())))
        .collect()
}

fn unreadable(part: &str) -> String {
    format!("has a filter that cannot be read: \"{part}\"")
}

fn parse_call(part: &str) -> Option<FilterCall> {
    let (name, arguments) = match part.find('(') {
        None => (part, Vec::new()),
        Some(open) => {
            let inside = part[open + 1..].strip_suffix(')')?;
            let arguments = if inside.trim().is_empty() {
                Vec::new()
            } else {
                split_outside_quotes(inside, ',')
                    .into_iter()
                    .map(|argument| argument_of(argument.trim()))
                    .collect::<Option<Vec<_>>>()?
            };
            (part[..open].trim(), arguments)
        }
    };
    let mut characters = name.chars();
    let valid = characters
        .next()
        .is_some_and(|first| first.is_ascii_lowercase())
        && characters.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    valid.then(|| FilterCall {
        name: name.to_string(),
        names: arguments
            .iter()
            .enumerate()
            .filter_map(|(position, (_, name))| name.clone().map(|name| (position, name)))
            .collect(),
        arguments: arguments.into_iter().map(|(value, _)| value).collect(),
    })
}

fn argument_of(text: &str) -> Option<(Value, Option<String>)> {
    match literal(text) {
        Some(value) => Some((value, None)),
        None => valid_name(text).then(|| (Value::Null, Some(text.to_string()))),
    }
}

fn literal(text: &str) -> Option<Value> {
    if let Some(inner) = text
        .strip_prefix('\'')
        .and_then(|rest| rest.strip_suffix('\''))
    {
        return Some(Value::String(inner.to_string()));
    }
    match serde_json::from_str::<Value>(text).ok()? {
        value @ (Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null) => Some(value),
        _ => None,
    }
}

fn split_outside_quotes(text: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut start = 0;
    for (index, character) in text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match (quote, character) {
            (Some('"'), '\\') => escaped = true,
            (Some(open), close) if open == close => quote = None,
            (None, '"' | '\'') => quote = Some(character),
            (None, found) if found == separator => {
                parts.push(&text[start..index]);
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&text[start..]);
    parts
}
