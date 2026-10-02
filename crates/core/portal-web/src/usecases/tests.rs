use portal_model::Language;
use toml_edit::DocumentMut;

use super::InterfaceOfDocument;

fn document(text: &str) -> DocumentMut {
    text.parse().unwrap()
}

#[test]
fn a_lone_main_file_gives_its_default_language() {
    let read = InterfaceOfDocument.run(&document("[interface]\ndefault_language = \"es\"\n"));
    assert_eq!(read.unwrap(), Language::parse("es").unwrap());
}

#[test]
fn a_lone_main_file_without_the_section_gives_english() {
    assert_eq!(
        InterfaceOfDocument.run(&document("")).unwrap(),
        Language::parse("en").unwrap()
    );
}

#[test]
fn an_unsupported_language_is_refused_by_its_key() {
    let errors = InterfaceOfDocument
        .run(&document("[interface]\ndefault_language = \"de\"\n"))
        .unwrap_err();
    assert_eq!(errors[0].field, "interface.default_language");
}
