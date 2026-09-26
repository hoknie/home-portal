use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use toml_edit::{DocumentMut, Item};

use crate::helpers::{
    holds_secrets, include_paths, merge, parse_document, refuse_if_readable, refuse_nested,
    take_secrets,
};
use crate::types::{ConfigError, Loaded, Revision, Snapshot, Source};

pub const CONFIGURATION_SECTION: &str = "configuration";
pub const WRITES_TO_KEY: &str = "writes_to";

pub fn load(main: &Path) -> Result<Loaded, ConfigError> {
    let mut sources = vec![read_source(main)?];
    for path in include_paths(main, &sources[0].document)? {
        let source = read_source(&path)?;
        refuse_nested(&path, &source.document)?;
        sources.push(source);
    }
    for source in &sources {
        if holds_secrets(&source.document) {
            refuse_if_readable(&source.path)?;
        }
    }
    let (mut merged, origins) = merge(&sources)?;
    let secrets = take_secrets(&mut merged);
    let revision = revision_of(&sources);
    let snapshot = Snapshot {
        document: Arc::new(merged),
        revision,
        origins: Arc::new(origins),
    };
    check_writes_to(main, &sources, &snapshot.document)?;
    Ok(Loaded {
        sources,
        snapshot,
        secrets,
    })
}

pub fn revision_of(sources: &[Source]) -> Revision {
    let mut bytes = Vec::new();
    for source in sources {
        bytes.extend_from_slice(source.path.as_os_str().as_encoded_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&source.bytes);
        bytes.push(0);
    }
    Revision::of(&bytes)
}

pub fn writes_to(main: &Path, document: &DocumentMut) -> PathBuf {
    document
        .get(CONFIGURATION_SECTION)
        .and_then(Item::as_table)
        .and_then(|table| table.get(WRITES_TO_KEY))
        .and_then(Item::as_str)
        .map(|name| main.parent().unwrap_or(Path::new(".")).join(name))
        .unwrap_or_else(|| main.to_path_buf())
}

fn check_writes_to(
    main: &Path,
    sources: &[Source],
    document: &DocumentMut,
) -> Result<(), ConfigError> {
    let target = writes_to(main, document);
    if sources.iter().any(|source| source.path == target) {
        return Ok(());
    }
    Err(ConfigError::Include {
        path: target,
        message: format!(
            "{CONFIGURATION_SECTION}.{WRITES_TO_KEY} must name the main file or one of the included files"
        ),
    })
}

fn read_source(path: &Path) -> Result<Source, ConfigError> {
    let bytes = fs::read(path).map_err(|source| match source.kind() {
        ErrorKind::NotFound => ConfigError::Missing {
            path: path.to_path_buf(),
            stray: None,
        },
        _ => ConfigError::Unreadable {
            path: path.to_path_buf(),
            source,
        },
    })?;
    let document = parse_document(&bytes).map_err(|message| ConfigError::Syntax {
        path: path.to_path_buf(),
        message,
    })?;
    Ok(Source {
        path: path.to_path_buf(),
        document,
        bytes,
    })
}
