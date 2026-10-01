use super::support::{Portal, no_negative_ports, open, rewrite};
use crate::services::ConfigStore;
use crate::types::ConfigError;

#[test]
fn a_missing_file_is_refused_by_path() {
    let error = ConfigStore::open("/nonexistent/home-portal.toml")
        .err()
        .unwrap();
    assert!(matches!(error, ConfigError::Missing { .. }));
    assert!(error.to_string().contains("/nonexistent/home-portal.toml"));
}

#[test]
fn a_syntax_error_is_refused_with_its_file_and_line() {
    let error = open(&[("home-portal.toml", ""), ("services.toml", "b = \n")])
        .err()
        .unwrap();
    assert!(matches!(error, ConfigError::Syntax { .. }));
    let message = error.to_string();
    assert!(
        message.contains("services.toml") && message.contains("line 1"),
        "{message}"
    );
}

#[test]
fn a_section_in_the_main_file_is_refused_naming_its_home() {
    let error = open(&[
        ("home-portal.toml", "[[services]]\nid = \"a\"\n"),
        ("services.toml", "[[services]]\nid = \"b\"\n"),
    ])
    .err()
    .unwrap();
    let message = error.to_string();
    assert!(
        message.contains("services: is in home-portal.toml; it belongs in services.toml"),
        "{message}"
    );
}

#[test]
fn include_and_configuration_are_refused_by_key() {
    for (key, text) in [
        ("include", "include = [\"extra.toml\"]\n"),
        (
            "configuration",
            "[configuration]\nwrites_to = \"services.toml\"\n",
        ),
    ] {
        let message = open(&[("home-portal.toml", text)])
            .err()
            .unwrap()
            .to_string();
        assert!(
            message.contains(&format!("{key}: is no longer read")),
            "{message}"
        );
    }
}

#[test]
fn a_key_set_by_two_files_names_both() {
    let error = open(&[
        ("home-portal.toml", "[network]\nport = 8080\n"),
        ("services.toml", "[network]\nport = 9090\n"),
    ])
    .err()
    .unwrap();
    let message = error.to_string();
    assert!(message.contains("network.port"), "{message}");
    assert!(
        message.contains("home-portal.toml") && message.contains("services.toml"),
        "{message}"
    );
}

#[test]
fn a_home_shared_by_two_sections_holds_both_and_names_itself_as_their_origin() {
    let portal = Portal::with(&[
        (
            "home-portal.toml",
            "[files]\nautomations = \"rules.toml\"\nwebhooks = \"rules.toml\"\n",
        ),
        (
            "rules.toml",
            "[[automations]]\nid = \"a\"\n\n[[webhooks]]\nid = \"w\"\n",
        ),
    ]);
    let snapshot = portal.store.read();
    assert_eq!(
        snapshot.origins.of("automations", 0).unwrap(),
        portal.path("rules.toml")
    );
    assert_eq!(
        snapshot.origins.of("webhooks", 0).unwrap(),
        portal.path("rules.toml")
    );
}

#[test]
fn every_entry_knows_the_file_it_came_from() {
    let portal = Portal::with(&[
        ("home-portal.toml", "[network]\nport = 8080\n"),
        ("services.toml", "[[services]]\nid = \"b\"\n"),
    ]);
    super::support::rewrite(
        &portal.path("services.toml"),
        &format!(
            "{}\n[[services]]\nid = \"by-hand\"\n",
            portal.text("services.toml")
        ),
    );
    let snapshot = portal.store.read();
    assert_eq!(
        snapshot.origins.of("services", 1).unwrap(),
        portal.path("services.toml")
    );
    assert_eq!(
        snapshot.origins.table("network").unwrap(),
        portal.path("home-portal.toml")
    );
    assert_eq!(snapshot.origins.count("services"), 2);
}

#[test]
fn the_revision_covers_every_file() {
    let portal = Portal::with(&[
        ("home-portal.toml", ""),
        ("services.toml", "[[services]]\nid = \"a\"\n"),
    ]);
    let before = portal.store.read().revision;
    rewrite(&portal.path("services.toml"), "[[services]]\nid = \"b\"\n");
    assert_ne!(portal.store.read().revision, before);
}

#[test]
fn a_value_that_fails_a_feature_validator_is_refused_by_key() {
    let portal = Portal::with(&[("home-portal.toml", "port = -1\n")]);
    let error = portal.store.adopt(vec![no_negative_ports]).unwrap_err();
    assert!(
        error.to_string().contains("port: must not be negative"),
        "{error}"
    );
}

#[test]
fn an_edit_made_by_hand_is_seen_and_a_broken_one_is_kept_out() {
    let portal = Portal::with(&[("home-portal.toml", "port = 1\n")]);
    rewrite(&portal.path("home-portal.toml"), "port = 2\n");
    assert_eq!(portal.store.read().document["port"].as_integer(), Some(2));
    rewrite(&portal.path("home-portal.toml"), "port = \n");
    assert_eq!(portal.store.read().document["port"].as_integer(), Some(2));
    assert!(portal.store.problem().is_some());
}

fn slow_to_check(document: &toml_edit::DocumentMut) -> Vec<portal_feature::FieldError> {
    if document.get("slow").is_some() {
        std::thread::sleep(std::time::Duration::from_millis(400));
    }
    Vec::new()
}

#[test]
fn a_reader_arriving_while_another_reloads_is_answered_at_once_with_the_last_adopted_content() {
    let portal = super::support::Portal::with(&[("home-portal.toml", "name = \"old\"\n")]);
    portal.store.adopt(vec![slow_to_check]).unwrap();
    super::support::rewrite(
        &portal.path("home-portal.toml"),
        "name = \"new\"\nslow = true\n",
    );
    let first = portal.store.clone();
    let reloading = std::thread::spawn(move || first.read());
    std::thread::sleep(std::time::Duration::from_millis(100));
    let asked = std::time::Instant::now();
    let arriving = portal.store.read();
    assert!(asked.elapsed() < std::time::Duration::from_millis(200));
    let name = |snapshot: &crate::Snapshot| {
        snapshot
            .document
            .get("name")
            .and_then(|item| item.as_str())
            .map(str::to_string)
    };
    assert_eq!(name(&arriving).as_deref(), Some("old"));
    assert_eq!(name(&reloading.join().unwrap()).as_deref(), Some("new"));
    assert_eq!(name(&portal.store.read()).as_deref(), Some("new"));
}

fn panics_on_boom(document: &toml_edit::DocumentMut) -> Vec<portal_feature::FieldError> {
    assert!(document.get("boom").is_none(), "a validator broke");
    Vec::new()
}

#[test]
fn a_loader_that_panics_keeps_the_last_good_configuration_and_later_edits_still_load() {
    let portal = super::support::Portal::with(&[("home-portal.toml", "port = 1\n")]);
    portal.store.adopt(vec![panics_on_boom]).unwrap();
    super::support::rewrite(&portal.path("home-portal.toml"), "port = 2\nboom = true\n");
    assert_eq!(portal.store.read().document["port"].as_integer(), Some(1));
    assert!(portal.store.problem().is_some());
    super::support::rewrite(&portal.path("home-portal.toml"), "port = 3\n");
    let other = portal.store.clone();
    let from_another_thread = std::thread::spawn(move || other.read());
    assert_eq!(
        from_another_thread.join().unwrap().document["port"].as_integer(),
        Some(3)
    );
}
