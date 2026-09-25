use std::fs;
use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::{StatusChange, StatusObserver};
use tempfile::TempDir;

use super::{Outbox, TelegramNotifier, check_telegram};
use crate::types::{Outgoing, TelegramSection};

const ENABLED: &str = "[secrets]\ntelegram_token = \"abc\"\n\n[notifications.telegram]\nenabled = true\nsecret = \"telegram_token\"\nchat_id = \"42\"\n";

fn store(text: &str) -> (TempDir, Arc<ConfigStore>) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    #[cfg(unix)]
    if text.contains("[secrets]") {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    }
    (directory, Arc::new(ConfigStore::open(&path).unwrap()))
}

fn change(was: &str, now: &str) -> StatusChange {
    StatusChange {
        service: "nas".into(),
        name: "NAS".into(),
        was: was.into(),
        now: now.into(),
        error: (now == "down").then(|| "connection refused".to_string()),
        diagnosis: None,
        notify: true,
    }
}

#[test]
fn a_failure_and_a_recovery_are_announced_and_the_first_up_is_not() {
    let (_directory, configuration) = store(ENABLED);
    let outbox = Arc::new(Outbox::default());
    let notifier = TelegramNotifier::new(configuration, outbox.clone());
    notifier.changed(&change("unknown", "up"));
    assert_eq!(
        outbox.waiting(),
        0,
        "a portal that just started says nothing about a service that works"
    );
    notifier.changed(&change("up", "down"));
    notifier.changed(&change("down", "up"));
    assert_eq!(outbox.waiting(), 2);
    let first = outbox.take().unwrap();
    assert_eq!(first.chat_id, "42");
    assert!(
        first.text.contains("NAS") && first.text.contains("up → down"),
        "{}",
        first.text
    );
    assert!(first.text.contains("connection refused"), "{}", first.text);
}

#[test]
fn a_failure_at_start_up_is_announced() {
    let (_directory, configuration) = store(ENABLED);
    let outbox = Arc::new(Outbox::default());
    TelegramNotifier::new(configuration, outbox.clone()).changed(&change("unknown", "down"));
    assert_eq!(outbox.waiting(), 1);
}

#[test]
fn nothing_is_announced_while_the_section_is_off() {
    let (_directory, configuration) = store("[notifications.telegram]\nchat_id = \"42\"\n");
    let outbox = Arc::new(Outbox::default());
    TelegramNotifier::new(configuration, outbox.clone()).changed(&change("up", "down"));
    assert_eq!(outbox.waiting(), 0);
}

#[test]
fn the_states_and_recovered_settings_choose_what_is_announced() {
    let (_directory, configuration) = store(&format!(
        "{ENABLED}states = [\"unreadable\"]\nrecovered = false\n"
    ));
    let settings = TelegramSection::read(&configuration.read().document).unwrap();
    assert!(settings.announces("unreadable", false));
    assert!(!settings.announces("down", false));
    assert!(!settings.announces("up", false));
}

#[test]
fn the_queue_keeps_the_newest_hundred_and_counts_what_it_dropped() {
    let outbox = Outbox::default();
    for index in 0..Outbox::CAPACITY + 5 {
        outbox.push(Outgoing {
            chat_id: "42".into(),
            text: index.to_string(),
        });
    }
    assert_eq!(outbox.waiting(), Outbox::CAPACITY);
    assert_eq!(outbox.dropped(), 5);
    assert_eq!(outbox.take().unwrap().text, "5");
}

#[test]
fn enabling_telegram_without_a_token_or_a_chat_names_the_key() {
    let (_directory, configuration) = store("[notifications.telegram]\nenabled = true\n");
    let fields: Vec<String> = check_telegram(&configuration.read().document, &configuration)
        .into_iter()
        .map(|error| error.field)
        .collect();
    assert_eq!(
        fields,
        vec![
            "notifications.telegram.secret",
            "notifications.telegram.chat_id"
        ]
    );
    let (_directory, configuration) =
        store("[notifications.telegram]\nenabled = true\nsecret = \"nope\"\nchat_id = \"42\"\n");
    let errors = check_telegram(&configuration.read().document, &configuration);
    assert!(errors[0].message.contains("nope"), "{}", errors[0].message);
    assert!(!errors[0].message.contains("abc"));
}

#[test]
fn a_change_of_a_service_that_does_not_notify_is_not_announced() {
    let (_directory, configuration) = store(ENABLED);
    let outbox = Arc::new(Outbox::default());
    let notifier = TelegramNotifier::new(configuration, outbox.clone());
    notifier.changed(&StatusChange {
        notify: false,
        ..change("up", "down")
    });
    assert_eq!(outbox.waiting(), 0);
}
