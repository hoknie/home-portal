use std::fs;
use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::{Notification, StatusChange};
use tempfile::TempDir;

use super::Outbox;
use crate::types::Rules;

pub fn store(text: &str) -> (TempDir, Arc<ConfigStore>) {
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

pub fn change(was: &str, now: &str) -> StatusChange {
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
fn the_rules_default_to_failures_and_recoveries() {
    let rules = Rules::read(&"".parse().unwrap()).unwrap();
    assert!(rules.announces("down", false));
    assert!(rules.announces("unreadable", false));
    assert!(rules.announces("up", false));
    assert!(!rules.announces("up", true));
    assert!(!rules.announces("degraded", false));
}

#[test]
fn a_file_written_before_channels_existed_keeps_its_rules_and_the_new_place_wins() {
    let legacy =
        "[notifications.telegram]\nenabled = true\nstates = [\"down\"]\nrecovered = false\n";
    let rules = Rules::read(&legacy.parse().unwrap()).unwrap();
    assert_eq!(rules.states, vec!["down"]);
    assert!(!rules.recovered);
    let both = format!("[notifications]\nstates = [\"unreadable\"]\n\n{legacy}");
    let rules = Rules::read(&both.parse().unwrap()).unwrap();
    assert_eq!(rules.states, vec!["unreadable"]);
    assert!(!rules.recovered);
}

#[test]
fn the_queue_keeps_the_newest_hundred_and_counts_what_it_dropped() {
    let outbox = Outbox::default();
    for index in 0..Outbox::CAPACITY + 5 {
        outbox.push("telegram", Notification::new("", index.to_string()));
    }
    assert_eq!(outbox.waiting(), Outbox::CAPACITY);
    assert_eq!(outbox.dropped(), 5);
    assert_eq!(outbox.take().unwrap().text, "5");
}

#[test]
fn a_state_outside_the_vocabulary_is_refused_by_its_key() {
    let problems = Rules::problems(
        &"[notifications]\nstates = [\"down\", \"broken\"]\n"
            .parse()
            .unwrap(),
    );
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].field, "notifications.states");
    assert!(problems[0].message.contains("broken"));
}
