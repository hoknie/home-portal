use std::path::Path;

use portal_feature::FieldError;
use toml_edit::DocumentMut;

use crate::types::{Layout, Section};

pub const RETIRED_KEYS: [&str; 2] = ["include", "configuration"];
pub const RETIRED: &str =
    "is no longer read; every section lives in its own file beside the main file";

fn shown(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .display()
        .to_string()
}

pub fn misplaced(
    path: &Path,
    document: &DocumentMut,
    main: &Path,
    layout: &Layout,
) -> Vec<FieldError> {
    let base = main.parent().unwrap_or(Path::new(""));
    let mut problems = Vec::new();
    for (key, _) in document.iter() {
        if RETIRED_KEYS.contains(&key) {
            problems.push(FieldError::new(key, RETIRED));
            continue;
        }
        let Some(section) = Section::of_table(key) else {
            continue;
        };
        let home = layout.file_of(section);
        if section.is_folder() || home != path {
            problems.push(FieldError::new(
                key,
                format!(
                    "is in {}; it belongs in {}",
                    shown(path, base),
                    shown(&home, base) + if section.is_folder() { "/" } else { "" }
                ),
            ));
        }
    }
    problems
}
