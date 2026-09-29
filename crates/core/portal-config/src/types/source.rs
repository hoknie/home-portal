use std::path::PathBuf;

use toml_edit::DocumentMut;

use super::Shape;

#[derive(Debug, Clone)]
pub struct Source {
    pub path: PathBuf,
    pub document: DocumentMut,
    pub bytes: Vec<u8>,
    pub shape: Shape,
}

impl Source {
    pub fn text(&self) -> String {
        match self.shape {
            Shape::Whole => self.document.to_string(),
            Shape::Entry => crate::helpers::unwrapped(&self.document).to_string(),
        }
    }
}
