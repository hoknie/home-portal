use std::path::{Component, Path};

use crate::types::RefusalCode;

pub const DEEPEST: usize = 2;

pub fn script_shape(script: &str) -> Option<(RefusalCode, &'static str)> {
    if script.trim().is_empty() {
        return Some((
            RefusalCode::Shape,
            "must name a script inside the scripts directory",
        ));
    }
    let path = Path::new(script);
    if path.is_absolute() {
        return Some((
            RefusalCode::Shape,
            "must be a path inside the scripts directory, not an absolute path",
        ));
    }
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            Component::CurDir => {}
            _ => {
                return Some((
                    RefusalCode::Shape,
                    "must stay inside the scripts directory, without ..",
                ));
            }
        }
    }
    if parts.iter().any(|part| part.starts_with('.')) {
        return Some((
            RefusalCode::Hidden,
            "must not be a hidden file or lie in a hidden folder",
        ));
    }
    if parts.len() > DEEPEST {
        return Some((
            RefusalCode::TooDeep,
            "must lie directly in the scripts directory or in one of its subfolders, not deeper",
        ));
    }
    None
}

pub fn script_shape_problem(script: &str) -> Option<&'static str> {
    script_shape(script).map(|(_, message)| message)
}
