use portal_feature::FieldError;
use portal_model::Language;
use toml_edit::DocumentMut;

use crate::services::read_interface;

#[derive(Debug, Clone, Copy, Default)]
pub struct InterfaceOfDocument;

impl InterfaceOfDocument {
    pub fn run(&self, document: &DocumentMut) -> Result<Language, Vec<FieldError>> {
        read_interface(document)
    }
}
