use std::sync::Arc;

use toml_edit::DocumentMut;

use super::{Origins, Revision};

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub document: Arc<DocumentMut>,
    pub revision: Revision,
    pub origins: Arc<Origins>,
}
