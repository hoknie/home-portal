use std::fs;

use super::support::{Portal, open, rewrite};
use crate::types::Section;

fn error_of(files: &[(&str, &str)]) -> String {
    open(files)
        .err()
        .map(|error| error.to_string())
        .unwrap_or_default()
}

#[test]
fn default_homes_are_read_beside_the_main_file() {
    let portal = Portal::with(&[
        ("home-portal.toml", "[network]\nport = 8080\n"),
        (
            "services.toml",
            "[[services]]\nid = \"a\"\n\n[[services]]\nid = \"b\"\n",
        ),
    ]);
    fs::create_dir(portal.path("workflows")).unwrap();
    fs::write(
        portal.path("workflows/revive.toml"),
        "id = \"revive\"\ntitle = \"Revive\"\n",
    )
    .unwrap();
    fs::write(portal.path("workflows/.hidden.toml"), "id = \"hidden\"\n").unwrap();
    fs::write(portal.path("workflows/notes.md"), "# not a workflow\n").unwrap();
    let snapshot = portal.store.read();
    assert_eq!(
        snapshot.document["services"]
            .as_array_of_tables()
            .unwrap()
            .len(),
        2
    );
    let workflows = snapshot.document["workflows"].as_array_of_tables().unwrap();
    assert_eq!(workflows.len(), 1);
    assert_eq!(workflows.get(0).unwrap()["id"].as_str(), Some("revive"));
}

#[test]
fn a_home_of_the_owners_choice_is_read_and_written() {
    let portal = Portal::with(&[(
        "home-portal.toml",
        "[files]\nautomations = \"rules.toml\"\nwebhooks = \"rules.toml\"\nworkflows = \"rules/flows/\"\n",
    )]);
    assert_eq!(
        portal.store.home_of(Section::Automations),
        portal.path("rules.toml")
    );
    assert_eq!(
        portal.store.home_of(Section::Webhooks),
        portal.path("rules.toml")
    );
    assert_eq!(
        portal.store.workflow_file("x"),
        portal.path("rules/flows/x.toml")
    );
}

#[test]
fn a_bad_home_is_named_under_files() {
    for (files, key) in [
        ("services = \"../shared/services.toml\"", "files.services"),
        ("services = \"/etc/services.toml\"", "files.services"),
        ("services = \"home-portal.toml\"", "files.services"),
        ("services = \"lists/\"", "files.services"),
        ("colours = \"colours.toml\"", "files.colours"),
        ("users = \"workflows/users.toml\"", "files.users"),
    ] {
        let message = error_of(&[("home-portal.toml", &format!("[files]\n{files}\n"))]);
        assert!(message.contains(key), "{files}: {message}");
    }
}

#[test]
fn a_changed_home_takes_effect_only_after_a_new_start() {
    let portal = Portal::with(&[("home-portal.toml", "[network]\nport = 8080\n")]);
    rewrite(
        &portal.path("home-portal.toml"),
        "[files]\nservices = \"lists.toml\"\n\n[network]\nport = 8080\n",
    );
    portal.store.read();
    assert_eq!(
        portal.store.home_of(Section::Services),
        portal.path("services.toml")
    );
    let again = crate::ConfigStore::open(portal.path("home-portal.toml")).unwrap();
    assert_eq!(again.home_of(Section::Services), portal.path("lists.toml"));
}

#[test]
fn a_workflow_file_must_be_named_after_its_id_and_hold_only_a_workflow() {
    for (text, expected) in [
        ("id = \"restart\"\n", "restart"),
        ("title = \"No id\"\n", "id"),
        ("id = \"revive\"\ncolour = \"red\"\n", "colour"),
    ] {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join("home-portal.toml"), "").unwrap();
        fs::create_dir(directory.path().join("workflows")).unwrap();
        fs::write(directory.path().join("workflows/revive.toml"), text).unwrap();
        let message = crate::ConfigStore::open(directory.path().join("home-portal.toml"))
            .err()
            .unwrap()
            .to_string();
        assert!(message.contains("workflows/revive.toml"), "{message}");
        assert!(message.contains(expected), "{message}");
    }
}

#[test]
fn a_problem_in_a_workflow_names_its_file() {
    let portal = Portal::with(&[("home-portal.toml", "")]);
    fs::create_dir(portal.path("workflows")).unwrap();
    fs::write(portal.path("workflows/a.toml"), "id = \"a\"\n").unwrap();
    fs::write(portal.path("workflows/b.toml"), "id = \"b\"\n").unwrap();
    portal.store.read();
    let error = portal
        .store
        .adopt(vec![|document: &toml_edit::DocumentMut| {
            document["workflows"]
                .as_array_of_tables()
                .into_iter()
                .flat_map(|entries| entries.iter().enumerate())
                .filter(|(_, entry)| entry.get("title").is_none())
                .map(|(index, _)| {
                    portal_feature::FieldError::new(
                        format!("workflows[{index}].title"),
                        "is required",
                    )
                })
                .collect()
        }])
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("workflows[1].title: is required (in workflows/b.toml)"),
        "{error}"
    );
}

#[test]
fn two_workflow_files_keep_their_own_steps_when_the_merged_document_is_read_back() {
    let portal = Portal::with(&[("home-portal.toml", "")]);
    fs::create_dir(portal.path("workflows")).unwrap();
    fs::write(
        portal.path("workflows/a.toml"),
        "id = \"a\"\n\n[[steps]]\nid = \"first\"\n",
    )
    .unwrap();
    fs::write(
        portal.path("workflows/b.toml"),
        "id = \"b\"\n\n[[steps]]\nid = \"second\"\n",
    )
    .unwrap();
    let text = portal.store.read().document.to_string();
    let reread: toml_edit::DocumentMut = text.parse().unwrap();
    let workflows = reread["workflows"].as_array_of_tables().unwrap();
    for (index, step) in [(0, "first"), (1, "second")] {
        let steps = workflows.get(index).unwrap()["steps"]
            .as_array_of_tables()
            .unwrap();
        assert_eq!(steps.len(), 1, "{text}");
        assert_eq!(steps.get(0).unwrap()["id"].as_str(), Some(step));
    }
}
