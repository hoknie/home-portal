use toml_edit::value;

use super::support::{Portal, open};
use crate::types::{ConfigError, SecretString};

#[test]
fn secrets_are_read_but_never_appear_in_the_document() {
    let portal = Portal::with(&[
        ("home-portal.toml", "include = [\"secrets.toml\"]\n"),
        ("secrets.toml", "[secrets]\ntelegram_token = \"abc123\"\n"),
    ]);
    assert!(portal.store.read().document.get("secrets").is_none());
    assert_eq!(
        portal.store.secret("telegram_token").unwrap().expose(),
        "abc123"
    );
    assert!(portal.store.secret("nope").is_none());
    assert_eq!(
        portal.store.secret_names(),
        vec!["telegram_token".to_string()]
    );
}

#[cfg(unix)]
#[test]
fn a_secrets_file_anyone_can_read_is_refused() {
    let error = open(&[
        ("home-portal.toml", "include = [\"secrets.toml\"]\n"),
        ("secrets.toml", "[secrets]\ntoken = \"abc\"\n"),
    ])
    .err()
    .unwrap();
    assert!(matches!(error, ConfigError::Permissions { .. }), "{error}");
    let message = error.to_string();
    assert!(
        message.contains("secrets.toml") && message.contains("0600"),
        "{message}"
    );
}

#[test]
fn a_secret_never_prints_its_value() {
    let secret = SecretString::new("hunter2");
    assert_eq!(format!("{secret:?}"), "[secret]");
    assert_eq!(format!("{secret}"), "[secret]");
    assert!(!format!("{secret:?} {secret}").contains("hunter2"));
}

#[tokio::test]
async fn concurrent_writes_are_applied_one_after_the_other() {
    let portal = Portal::with(&[("home-portal.toml", "count = 0\n")]);
    let mut handles = Vec::new();
    for _ in 0..8 {
        let store = portal.store.clone();
        let target = portal.path("home-portal.toml");
        handles.push(tokio::spawn(async move {
            loop {
                let revision = store.read().revision;
                let outcome = store
                    .update(&target, &revision, |document| {
                        let count = document["count"].as_integer().unwrap_or_default();
                        document["count"] = value(count + 1);
                        Ok(())
                    })
                    .await;
                if outcome.is_ok() {
                    break;
                }
            }
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }
    assert_eq!(portal.store.read().document["count"].as_integer(), Some(8));
}

#[test]
fn a_secret_has_no_way_to_be_serialized() {
    let source = include_str!("../../types/secret_string.rs");
    assert!(
        !source.contains("Serialize"),
        "SecretString must not implement Serialize"
    );
    assert!(
        !source.contains("derive(serde"),
        "SecretString must not derive a serde trait"
    );
}

#[test]
fn a_check_that_reads_a_secret_while_a_hand_edit_is_reloaded_does_not_lock_the_store() {
    let portal = Portal::with(&[
        (
            "home-portal.toml",
            "include = [\"secrets.toml\"]\nname = \"old\"\n",
        ),
        ("secrets.toml", "[secrets]\ntoken = \"abc\"\n"),
    ]);
    super::support::private(&portal.path("secrets.toml"));
    let weak = std::sync::Arc::downgrade(&portal.store);
    portal
        .store
        .adopt_checks(vec![std::sync::Arc::new(
            move |document: &toml_edit::DocumentMut| {
                let store = weak.upgrade().unwrap();
                if document.get("wants").is_some() && store.secret("added").is_none() {
                    return vec![portal_feature::FieldError::new(
                        "wants",
                        "names a secret that is not set",
                    )];
                }
                Vec::new()
            },
        )])
        .unwrap();
    super::support::rewrite(
        &portal.path("secrets.toml"),
        "[secrets]\ntoken = \"abc\"\nadded = \"new\"\n",
    );
    super::support::private(&portal.path("secrets.toml"));
    super::support::rewrite(
        &portal.path("home-portal.toml"),
        "include = [\"secrets.toml\"]\nname = \"new\"\nwants = true\n",
    );
    let store = portal.store.clone();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(
            store
                .read()
                .document
                .get("name")
                .and_then(|item| item.as_str())
                .map(str::to_string),
        );
    });
    let name = receiver
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("reading the store after a hand edit hung");
    assert_eq!(name.as_deref(), Some("new"));
    assert_eq!(portal.store.secret("added").unwrap().expose(), "new");
}
