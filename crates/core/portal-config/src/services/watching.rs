use std::fs;
use std::path::{Path, PathBuf};

use crate::helpers::{layout_of, parse_document};
use crate::services::loading::workflow_files;

pub fn configuration_files(main: &Path) -> Vec<PathBuf> {
    let document = fs::read(main)
        .ok()
        .and_then(|bytes| parse_document(&bytes).ok())
        .unwrap_or_default();
    let layout = layout_of(main, &document);
    let mut files = vec![main.to_path_buf()];
    for path in layout.files() {
        if !files.contains(&path) {
            files.push(path);
        }
    }
    let folder = layout.folder();
    files.extend(workflow_files(&folder));
    files.push(folder);
    files
}
