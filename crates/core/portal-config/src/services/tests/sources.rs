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
    let error = open(&[
        ("home-portal.toml", "include = [\"a.toml\"]\n"),
        ("a.toml", "b = \n"),
    ])
    .err()
    .unwrap();
    assert!(matches!(error, ConfigError::Syntax { .. }));
    let message = error.to_string();
    assert!(
        message.contains("a.toml") && message.contains("line 1"),
        "{message}"
    );
}

#[test]
fn services_from_an_included_file_are_listed_with_the_others() {
    let portal = Portal::with(&[
        (
            "home-portal.toml",
            "include = [\"services.toml\"]\n\n[[services]]\nid = \"a\"\n",
        ),
        (
            "services.toml",
            "[[services]]\nid = \"b\"\n\n[[services]]\nid = \"c\"\n",
        ),
    ]);
    let snapshot = portal.store.read();
    let ids: Vec<&str> = snapshot.document["services"]
        .as_array_of_tables()
        .unwrap()
        .iter()
        .map(|table| table["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["a", "b", "c"]);
    assert!(snapshot.document.get("include").is_none());
}

#[test]
fn an_include_outside_the_directory_a_missing_file_and_a_nested_include_are_refused() {
    let outside = open(&[("home-portal.toml", "include = [\"../secrets.toml\"]\n")])
        .err()
        .unwrap();
    assert!(outside.to_string().contains("secrets.toml"), "{outside}");
    let missing = open(&[("home-portal.toml", "include = [\"nope.toml\"]\n")])
        .err()
        .unwrap();
    assert!(missing.to_string().contains("does not exist"), "{missing}");
    let nested = open(&[
        ("home-portal.toml", "include = [\"a.toml\"]\n"),
        ("a.toml", "include = [\"b.toml\"]\n"),
        ("b.toml", "x = 1\n"),
    ])
    .err()
    .unwrap();
    assert!(nested.to_string().contains("may not include"), "{nested}");
}

#[test]
fn a_key_set_by_two_files_names_both() {
    let error = open(&[
        (
            "home-portal.toml",
            "include = [\"net.toml\"]\n\n[network]\nport = 8080\n",
        ),
        ("net.toml", "[network]\nport = 9090\n"),
    ])
    .err()
    .unwrap();
    let message = error.to_string();
    assert!(message.contains("network.port"), "{message}");
    assert!(
        message.contains("home-portal.toml") && message.contains("net.toml"),
        "{message}"
    );
}

#[test]
fn nested_tables_from_several_files_merge_and_their_entries_concatenate() {
    let portal = Portal::with(&[
        (
            "home-portal.toml",
            "include = [\"widgets.toml\"]\n\n[[dashboard.widgets]]\ntype = \"status-summary\"\n",
        ),
        (
            "widgets.toml",
            "[[dashboard.widgets]]\ntype = \"weather\"\n",
        ),
    ]);
    let snapshot = portal.store.read();
    let kinds: Vec<&str> = snapshot.document["dashboard"]["widgets"]
        .as_array_of_tables()
        .unwrap()
        .iter()
        .map(|table| table["type"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, vec!["status-summary", "weather"]);
    assert_eq!(
        snapshot
            .origins
            .of("dashboard.widgets", 1)
            .unwrap()
            .file_name()
            .unwrap(),
        "widgets.toml"
    );
}

#[test]
fn every_entry_knows_the_file_it_came_from() {
    let portal = Portal::with(&[
        (
            "home-portal.toml",
            "include = [\"services.toml\"]\n\n[[services]]\nid = \"a\"\n\n[network]\nport = 8080\n",
        ),
        ("services.toml", "[[services]]\nid = \"b\"\n"),
    ]);
    let snapshot = portal.store.read();
    assert_eq!(
        snapshot.origins.of("services", 0).unwrap(),
        portal.path("home-portal.toml")
    );
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
        ("home-portal.toml", "include = [\"services.toml\"]\n"),
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
fn a_reader_arriving_while_another_reloads_a_hand_edit_gets_the_new_content() {
    let portal = super::support::Portal::with(&[("home-portal.toml", "name = \"old\"\n")]);
    portal.store.adopt(vec![slow_to_check]).unwrap();
    super::support::rewrite(
        &portal.path("home-portal.toml"),
        "name = \"new\"\nslow = true\n",
    );
    let first = portal.store.clone();
    let reloading = std::thread::spawn(move || first.read());
    std::thread::sleep(std::time::Duration::from_millis(100));
    let arriving = portal.store.read();
    let name = |snapshot: &crate::Snapshot| {
        snapshot
            .document
            .get("name")
            .and_then(|item| item.as_str())
            .map(str::to_string)
    };
    assert_eq!(name(&arriving).as_deref(), Some("new"));
    assert_eq!(name(&reloading.join().unwrap()).as_deref(), Some("new"));
}
