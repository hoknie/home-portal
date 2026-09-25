use portal_model::Language;
use toml_edit::DocumentMut;

use super::{DEFAULT_LANGUAGE, read_interface, validate_interface};

fn document(text: &str) -> DocumentMut {
    text.parse().unwrap()
}

#[test]
fn without_the_section_the_default_language_is_english() {
    assert_eq!(read_interface(&document("")), Ok(Language::En));
}

#[test]
fn a_supported_default_is_read() {
    assert_eq!(
        read_interface(&document("[interface]\ndefault_language = \"ru\"\n")),
        Ok(Language::Ru)
    );
}

#[test]
fn an_unsupported_default_is_refused_by_name() {
    let errors = validate_interface(&document("[interface]\ndefault_language = \"de\"\n"));
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field, DEFAULT_LANGUAGE);
}
