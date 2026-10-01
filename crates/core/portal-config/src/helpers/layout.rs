use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use portal_feature::FieldError;
use toml_edit::{DocumentMut, Item};

use crate::types::{Home, Layout, Section};

pub const FILES_SECTION: &str = "files";

fn base(main: &Path) -> PathBuf {
    main.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .to_path_buf()
}

fn written(document: &DocumentMut, key: &str) -> Option<String> {
    document
        .get(FILES_SECTION)
        .and_then(Item::as_table_like)
        .and_then(|table| table.get(key))
        .and_then(Item::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

pub fn layout_of(main: &Path, document: &DocumentMut) -> Layout {
    let base = base(main);
    let homes: BTreeMap<Section, Home> = Section::ALL
        .into_iter()
        .map(|section| {
            let chosen = written(document, section.key())
                .filter(|text| inside(&base, text))
                .unwrap_or_else(|| section.default_home().to_string());
            let path = base.join(chosen.trim_end_matches('/'));
            let home = if section.is_folder() {
                Home::Folder(path)
            } else {
                Home::File(path)
            };
            (section, home)
        })
        .collect();
    Layout::new(homes)
}

pub fn layout_errors(main: &Path, document: &DocumentMut) -> Vec<FieldError> {
    let Some(item) = document.get(FILES_SECTION) else {
        return Vec::new();
    };
    let Some(table) = item.as_table_like() else {
        return vec![FieldError::new(FILES_SECTION, "must be a table")];
    };
    let base = base(main);
    let layout = layout_of(main, document);
    let folder = layout.folder();
    let mut errors = Vec::new();
    for (key, value) in table.iter() {
        let field = format!("{FILES_SECTION}.{key}");
        let Some(section) = Section::of_key(key) else {
            errors.push(FieldError::new(
                field,
                format!(
                    "unknown key; expected one of {}",
                    Section::ALL.map(Section::key).join(", ")
                ),
            ));
            continue;
        };
        let Some(text) = value
            .as_str()
            .map(str::trim)
            .filter(|text| !text.is_empty())
        else {
            errors.push(FieldError::new(field, "must be a non-empty path"));
            continue;
        };
        if let Some(problem) = problem_of(section, text, &base, main, &folder) {
            errors.push(FieldError::new(field, problem));
        }
    }
    errors
}

fn problem_of(
    section: Section,
    text: &str,
    base: &Path,
    main: &Path,
    folder: &Path,
) -> Option<&'static str> {
    if Path::new(text).is_absolute() || !inside(base, text) {
        return Some("must be a path inside the configuration directory");
    }
    let path = base.join(text.trim_end_matches('/'));
    if path == main {
        return Some("must not be the main configuration file");
    }
    if section.is_folder() {
        return path
            .is_file()
            .then_some("must be a folder: workflows are kept one file each");
    }
    if text.ends_with('/') || path.is_dir() {
        return Some("must be a file, not a folder");
    }
    (path == folder || path.starts_with(folder)).then_some("must not lie in the workflow folder")
}

pub fn inside(directory: &Path, name: &str) -> bool {
    let candidate = Path::new(name);
    if candidate.is_absolute() {
        return false;
    }
    let mut depth = 0i32;
    for part in candidate.components() {
        match part {
            std::path::Component::ParentDir => depth -= 1,
            std::path::Component::CurDir => {}
            _ => depth += 1,
        }
        if depth < 0 {
            return false;
        }
    }
    directory.join(candidate).starts_with(directory)
}
