use std::path::PathBuf;

use toml_edit::DocumentMut;

#[derive(Debug, Clone)]
pub struct Source {
    pub path: PathBuf,
    pub document: DocumentMut,
    pub bytes: Vec<u8>,
}
