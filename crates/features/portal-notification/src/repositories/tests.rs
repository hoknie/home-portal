use toml_edit::DocumentMut;

use super::{channel_table, write_rules};
use crate::types::Rules;

#[test]
fn writing_the_rules_puts_them_in_notifications_and_keeps_the_channel_and_its_comments() {
    let mut document: DocumentMut = "# alerts\n[notifications.telegram]\nenabled = true # on\n"
        .parse()
        .unwrap();
    write_rules(
        &mut document,
        &Rules {
            states: vec!["down".into(), "unreadable".into()],
            recovered: true,
        },
    );
    let text = document.to_string();
    assert!(text.contains("# alerts"), "{text}");
    assert!(text.contains("enabled = true # on"), "{text}");
    assert!(
        text.contains("[notifications]\nstates = [\"down\", \"unreadable\"]\nrecovered = true"),
        "{text}"
    );
    assert_eq!(text.matches("states").count(), 1, "{text}");
    let rules = Rules::read(&text.parse().unwrap()).unwrap();
    assert_eq!(rules.states, vec!["down", "unreadable"]);
}

#[test]
fn a_channel_table_is_created_under_notifications_when_missing() {
    let mut document: DocumentMut = "".parse().unwrap();
    channel_table(&mut document, "telegram").insert("enabled", toml_edit::value(true));
    assert_eq!(
        document.to_string(),
        "[notifications.telegram]\nenabled = true\n"
    );
}
