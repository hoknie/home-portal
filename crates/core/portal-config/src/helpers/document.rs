use std::fs;
use std::path::Path;
use toml_edit::DocumentMut;

use crate::types::Stamp;

pub fn parse_document(bytes: &[u8]) -> Result<DocumentMut, String> {
    let text = std::str::from_utf8(bytes).map_err(|error| format!("not UTF-8: {error}"))?;
    text.parse::<DocumentMut>()
        .map_err(|error| error.to_string().trim().to_string())
}

pub fn stamp_of(path: &Path) -> Option<Stamp> {
    let metadata = fs::metadata(path).ok()?;
    Some(Stamp {
        modified: metadata.modified().ok()?,
        length: metadata.len(),
    })
}
