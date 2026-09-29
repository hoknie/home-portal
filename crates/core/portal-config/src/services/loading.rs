use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use portal_feature::FieldError;
use toml_edit::{DocumentMut, Item};

use crate::helpers::{
    entry_id, holds_secrets, include_paths, merge, parse_document, refuse_if_readable,
    refuse_nested, take_secrets, wrapped,
};
use crate::types::{ConfigError, Layout, Loaded, Revision, Section, Shape, Snapshot, Source};

pub const CONFIGURATION_SECTION: &str = "configuration";
pub const WRITES_TO_KEY: &str = "writes_to";
pub const WRITES_TO_IGNORED: &str =
    "configuration.writes_to is ignored; new entries go to the home of their section";

pub fn read_main(main: &Path) -> Result<DocumentMut, ConfigError> {
    Ok(read_source(main, Shape::Whole)?.document)
}

pub fn load(main: &Path, layout: &Layout) -> Result<Loaded, ConfigError> {
    let main_source = read_source(main, Shape::Whole)?;
    let includes = include_paths(main, &main_source.document)?;
    let mut sources = Vec::new();
    for path in layout.files() {
        if path != main && path.is_file() {
            let source = read_source(&path, Shape::Whole)?;
            refuse_nested(&path, &source.document)?;
            sources.push(source);
        }
    }
    for path in workflow_files(&layout.folder()) {
        sources.push(read_entry(&path)?);
    }
    sources.push(main_source);
    for path in includes {
        if sources.iter().any(|source| source.path == path) {
            continue;
        }
        let source = read_source(&path, Shape::Whole)?;
        refuse_nested(&path, &source.document)?;
        sources.push(source);
    }
    loaded_from(sources)
}

pub fn loaded_from(sources: Vec<Source>) -> Result<Loaded, ConfigError> {
    for source in &sources {
        if holds_secrets(&source.document) {
            refuse_if_readable(&source.path)?;
        }
    }
    let (mut merged, origins) = merge(&sources)?;
    let secrets = take_secrets(&mut merged);
    let revision = revision_of(&sources);
    Ok(Loaded {
        sources,
        snapshot: Snapshot {
            document: Arc::new(merged),
            revision,
            origins: Arc::new(origins),
        },
        secrets,
    })
}

pub fn workflow_files(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            !name.starts_with('.')
                && path
                    .extension()
                    .is_some_and(|extension| extension == Layout::WORKFLOW_EXTENSION)
        })
        .collect();
    files.sort();
    files
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

pub fn names_writes_to(document: &DocumentMut) -> bool {
    document
        .get(CONFIGURATION_SECTION)
        .and_then(Item::as_table_like)
        .is_some_and(|table| table.contains_key(WRITES_TO_KEY))
}

pub fn entry_problems(path: &Path, document: &DocumentMut) -> Vec<FieldError> {
    let mut problems: Vec<FieldError> = document
        .iter()
        .filter(|(key, _)| !Section::WORKFLOW_KEYS.contains(key))
        .map(|(key, _)| {
            FieldError::new(
                key,
                "is not a key of a workflow; a workflow file holds one workflow",
            )
        })
        .collect();
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_default();
    match entry_id(document) {
        None => problems.push(FieldError::new(
            "id",
            format!("is required, and must be {stem}, the file's name"),
        )),
        Some(id) if id != stem => problems.push(FieldError::new(
            "id",
            format!(
                "is {id}, but the file is named {stem}.toml; the id and the file name must match"
            ),
        )),
        Some(_) => {}
    }
    problems
}

fn read_entry(path: &Path) -> Result<Source, ConfigError> {
    let source = read_source(path, Shape::Entry)?;
    let problems = entry_problems(path, &source.document);
    if !problems.is_empty() {
        return Err(ConfigError::Invalid {
            path: path.to_path_buf(),
            errors: problems,
        });
    }
    Ok(Source {
        document: wrapped(&source.document),
        ..source
    })
}

fn read_source(path: &Path, shape: Shape) -> Result<Source, ConfigError> {
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
        shape,
    })
}
