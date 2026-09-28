use super::chains::parse_chain;
use crate::helpers::{CLOSE, OPEN};
use crate::types::Placeholder;

pub const NAMESPACES: [&str; 9] = [
    "event", "inputs", "vars", "steps", "loop", "secrets", "item", "index", "portal",
];
pub const BARE_NAMESPACES: [&str; 2] = ["item", "index"];
pub const BAR: char = '|';

pub fn placeholders_in(template: &str) -> Vec<Placeholder<'_>> {
    let mut found = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find(OPEN) {
        let after = &rest[start + OPEN.len()..];
        match placeholder_at(after) {
            Some(placeholder) => {
                rest = &after[placeholder.inner_length + CLOSE.len()..];
                found.push(placeholder);
            }
            None => rest = &rest[start + 1..],
        }
    }
    found
}

pub fn placeholder_at(text: &str) -> Option<Placeholder<'_>> {
    let end = text.find(CLOSE)?;
    let inner = &text[..end];
    let (name, filters) = match inner.find(BAR) {
        None => (inner, Ok(Vec::new())),
        Some(bar) => (inner[..bar].trim(), parse_chain(&inner[bar + 1..])),
    };
    valid_name(name).then_some(Placeholder {
        name,
        filters,
        inner_length: end,
    })
}

pub fn valid_name(name: &str) -> bool {
    let mut parts = name.split('.');
    let first = parts.next().unwrap_or_default();
    NAMESPACES.contains(&first)
        && (name.contains('.') || BARE_NAMESPACES.contains(&first))
        && parts.all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        })
}
