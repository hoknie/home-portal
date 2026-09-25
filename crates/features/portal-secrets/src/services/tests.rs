use toml_edit::DocumentMut;

use super::referenced_secrets;

fn names(text: &str) -> Vec<String> {
    let document: DocumentMut = text.parse().unwrap();
    referenced_secrets(&document).into_iter().collect()
}

#[test]
fn a_secret_named_anywhere_in_the_configuration_is_found() {
    let text = "[notifications.telegram]\nsecret = \"telegram_token\"\n\n[[dashboard.widgets]]\ntype = \"calendar\"\nsettings = { secret = \"calendar_password\" }\n";
    assert_eq!(names(text), vec!["calendar_password", "telegram_token"]);
}

#[test]
fn a_key_that_is_not_a_secret_reference_is_ignored() {
    let text = "[network]\nport = 8080\nsecretive = \"no\"\n";
    assert!(names(text).is_empty());
}
