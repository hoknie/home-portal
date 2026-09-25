use std::path::{Path, PathBuf};

use toml_edit::DocumentMut;

use crate::types::ConfigError;

pub const INCLUDE_KEY: &str = "include";
pub const NOT_A_LIST: &str = "must be a list of file names";
pub const OUTSIDE: &str = "must name a file in the configuration directory";
pub const NESTED: &str = "an included file may not include further files";

pub fn include_paths(main: &Path, document: &DocumentMut) -> Result<Vec<PathBuf>, ConfigError> {
    let Some(item) = document.get(INCLUDE_KEY) else {
        return Ok(Vec::new());
    };
    let array = item.as_array().ok_or_else(|| ConfigError::Include {
        path: main.to_path_buf(),
        message: NOT_A_LIST.to_string(),
    })?;
    let directory = main.parent().unwrap_or(Path::new(".")).to_path_buf();
    let mut paths = Vec::new();
    for value in array {
        let name = value.as_str().ok_or_else(|| ConfigError::Include {
            path: main.to_path_buf(),
            message: NOT_A_LIST.to_string(),
        })?;
        let candidate = directory.join(name);
        if !inside(&directory, name) {
            return Err(ConfigError::Include {
                path: candidate,
                message: OUTSIDE.to_string(),
            });
        }
        if !candidate.is_file() {
            return Err(ConfigError::Include {
                path: candidate.clone(),
                message: "does not exist".to_string(),
            });
        }
        paths.push(candidate);
    }
    Ok(paths)
}

pub fn refuse_nested(path: &Path, document: &DocumentMut) -> Result<(), ConfigError> {
    match document.get(INCLUDE_KEY) {
        Some(_) => Err(ConfigError::Include {
            path: path.to_path_buf(),
            message: NESTED.to_string(),
        }),
        None => Ok(()),
    }
}

fn inside(directory: &Path, name: &str) -> bool {
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
