use portal_config::deserialize_section;
use portal_feature::FieldError;
use portal_model::Language;
use toml_edit::DocumentMut;

use crate::types::RawInterfaceSection;

pub const SECTION: &str = "interface";
pub const DEFAULT_LANGUAGE: &str = "interface.default_language";

pub fn read_interface(document: &DocumentMut) -> Result<Language, Vec<FieldError>> {
    let section: RawInterfaceSection =
        deserialize_section(document).map_err(|message| vec![FieldError::new(SECTION, message)])?;
    match section.interface.default_language {
        None => Ok(Language::default()),
        Some(text) => Language::parse(&text)
            .ok_or_else(|| vec![FieldError::new(DEFAULT_LANGUAGE, Language::PROBLEM)]),
    }
}

pub fn validate_interface(document: &DocumentMut) -> Vec<FieldError> {
    read_interface(document).err().unwrap_or_default()
}
