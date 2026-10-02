use std::fs;
use std::path::PathBuf;

use portal_config::ConfigError;
use portal_model::Language;

use super::support::{HOST, LOCAL, invalid, main_of};
use crate::failures::board::FailureBoard;
use crate::failures::fingerprint::Fingerprint;
use crate::failures::main_file::MainFile;
use crate::failures::problems::{Problem, problems_of};
use crate::types::BootError;

#[test]
fn configuration_and_feature_failures_enter_the_failure_mode_and_the_rest_exits() {
    let path = PathBuf::from("/x/home-portal.toml");
    let entering = [
        invalid(&["users"]),
        BootError::Configuration(ConfigError::Syntax {
            path: path.clone(),
            message: "line 3".into(),
        }),
        BootError::Configuration(ConfigError::Permissions {
            path: path.clone(),
            mode: 0o644,
        }),
        BootError::Configuration(ConfigError::Merge {
            message: "twice".into(),
        }),
        BootError::Feature {
            name: "icons",
            message: "no catalogue".into(),
        },
    ];
    for error in &entering {
        assert!(error.enters_failure_mode(), "{error}");
    }
    let exiting = [
        BootError::Configuration(ConfigError::Missing { path, stray: None }),
        BootError::Configuration(ConfigError::NoConfigurationPath),
        BootError::Address {
            value: "nowhere".into(),
            variable: "HOME_PORTAL_ADDRESS",
        },
        BootError::Bind {
            address: HOST.parse().unwrap(),
            source: std::io::Error::other("in use"),
        },
    ];
    for error in &exiting {
        assert!(!error.enters_failure_mode(), "{error}");
    }
}

#[test]
fn an_invalid_configuration_gives_one_problem_per_field() {
    let problems = problems_of(&invalid(&[
        "proxy.enabled",
        "dns.enabled",
        "dns.https.enabled",
    ]));
    let fields: Vec<_> = problems
        .iter()
        .map(|problem| problem.field.clone().unwrap())
        .collect();
    assert_eq!(
        fields,
        ["proxy.enabled", "dns.enabled", "dns.https.enabled"]
    );
    assert!(problems.iter().all(|problem| {
        problem
            .file
            .as_deref()
            .is_some_and(|file| file.ends_with("home-portal.toml"))
    }));
    assert_eq!(problems[0].message, "is no longer read");
}

#[test]
fn a_syntax_error_is_one_problem_naming_its_file_and_line() {
    let problems = problems_of(&BootError::Configuration(ConfigError::Syntax {
        path: PathBuf::from("/x/services.toml"),
        message: "expected `]` at line 3".into(),
    }));
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].file.as_deref(), Some("/x/services.toml"));
    assert_eq!(problems[0].field, None);
    assert!(problems[0].message.contains("line 3"));
}

#[test]
fn a_feature_that_cannot_start_is_one_problem_without_a_file() {
    let problems = problems_of(&BootError::Feature {
        name: "icons",
        message: "no catalogue".into(),
    });
    assert_eq!(
        problems,
        [Problem {
            file: None,
            field: None,
            message: "the icons feature cannot start: no catalogue".into(),
        }]
    );
}

#[test]
fn a_broken_users_file_keeps_the_port_of_the_main_file() {
    let directory = tempfile::tempdir().unwrap();
    let main = directory.path().join("home-portal.toml");
    fs::write(&main, "[network]\nport = 9191\n").unwrap();
    fs::write(directory.path().join("users.toml"), "[[users\n").unwrap();
    assert_eq!(MainFile::read(&main).address.to_string(), "127.0.0.1:9191");
}

#[test]
fn a_main_file_that_does_not_parse_gives_every_default() {
    let directory = tempfile::tempdir().unwrap();
    let main = directory.path().join("home-portal.toml");
    fs::write(&main, "[network\n").unwrap();
    let read = MainFile::read(&main);
    assert_eq!(read.address.to_string(), "127.0.0.1:8080");
    assert!(read.trusted.is_empty());
    assert!(read.environments.is_none());
    assert_eq!(read.language, Language::default());
}

#[test]
fn a_broken_network_section_keeps_the_environments_and_the_language() {
    let read = main_of(&format!(
        "[network]\nport = 70000\n\n[interface]\ndefault_language = \"ru\"\n\n{LOCAL}"
    ));
    assert_eq!(read.address.to_string(), "127.0.0.1:8080");
    assert!(read.environments.is_some());
    assert_eq!(read.language, Language::parse("ru").unwrap());
}

#[test]
fn the_address_variable_wins_and_a_bad_one_is_refused() {
    let read = main_of("[network]\nport = 9191\n");
    let chosen = read
        .clone()
        .listening(Some("127.0.0.1:9090".into()))
        .unwrap();
    assert_eq!(chosen.address.to_string(), "127.0.0.1:9090");
    assert!(matches!(
        read.listening(Some("nowhere".into())),
        Err(BootError::Address { .. })
    ));
}

#[test]
fn the_fingerprint_changes_when_a_watched_file_changes_and_only_then() {
    let directory = tempfile::tempdir().unwrap();
    let main = directory.path().join("home-portal.toml");
    fs::write(&main, "").unwrap();
    let first = Fingerprint::of(&main);
    assert_eq!(Fingerprint::of(&main), first);
    fs::write(directory.path().join("users.toml"), "[[users]]\n").unwrap();
    let added = Fingerprint::of(&main);
    assert_ne!(added, first);
    fs::write(
        directory.path().join("users.toml"),
        "[[users]]\nname = \"a\"\n",
    )
    .unwrap();
    let edited = Fingerprint::of(&main);
    assert_ne!(edited, added);
    fs::remove_file(directory.path().join("users.toml")).unwrap();
    assert_ne!(Fingerprint::of(&main), edited);
    fs::write(directory.path().join("history.ndjson"), "{}\n").unwrap();
    assert_eq!(Fingerprint::of(&main), Fingerprint::of(&main));
}

#[test]
fn a_recheck_with_the_same_problems_says_nothing_changed() {
    let board = FailureBoard::new(problems_of(&invalid(&["users"])), MainFile::defaults());
    let before = board.current().checked;
    assert!(!board.checked(problems_of(&invalid(&["users"])), MainFile::defaults()));
    assert!(board.current().checked >= before);
    assert!(board.checked(problems_of(&invalid(&["groups"])), MainFile::defaults()));
    assert_eq!(board.current().problems[0].field.as_deref(), Some("groups"));
}
