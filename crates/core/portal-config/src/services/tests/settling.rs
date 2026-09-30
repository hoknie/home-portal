use std::fs;

use super::support::{Portal, open};
use crate::pending_moves;

const ALL_IN_ONE: &str = r#"[network]
port = 8080

# the media box
[[services]]
id = "media"
name = "Media"

# the storage
[[services]]
id = "nas"
name = "NAS"

[proxy]
portal_host = "portal.example.com"

# nightly
[[automations]]
id = "backup"
title = "Backup"

# revive a service
[[workflows]]
id = "revive"
title = "Revive"

[[workflows.steps]]
id = "probe"
kind = "probe"

[[workflows]]
id = "nightly"
title = "Nightly"
"#;

#[test]
fn an_all_in_one_file_is_split_at_the_first_start() {
    let portal = Portal::with(&[("home-portal.toml", ALL_IN_ONE)]);
    let main = portal.text("home-portal.toml");
    assert_eq!(main.trim(), "[network]\nport = 8080");
    let services = portal.text("services.toml");
    assert!(
        services.contains("# the media box\n[[services]]\nid = \"media\""),
        "{services}"
    );
    assert!(services.contains("# the storage"), "{services}");
    assert!(portal.text("proxy.toml").contains("portal_host"));
    assert!(portal.text("automations.toml").contains("# nightly"));
    let revive = portal.text("workflows/revive.toml");
    assert!(revive.contains("id = \"revive\""), "{revive}");
    assert!(revive.contains("[[steps]]"), "{revive}");
    assert!(portal.text("workflows/nightly.toml").contains("Nightly"));
    assert_eq!(portal.text("home-portal.toml.previous"), ALL_IN_ONE);
    let snapshot = portal.store.read();
    assert_eq!(
        snapshot.document["services"]
            .as_array_of_tables()
            .unwrap()
            .len(),
        2
    );
    let ids: Vec<&str> = snapshot.document["workflows"]
        .as_array_of_tables()
        .unwrap()
        .iter()
        .map(|entry| entry["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["nightly", "revive"]);
    assert!(
        pending_moves(&portal.path("home-portal.toml"))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn the_permissions_table_stays_in_the_main_file() {
    let text = format!("[permissions]\nrequest_at_start = false\n\n{ALL_IN_ONE}");
    let portal = Portal::with(&[("home-portal.toml", &text)]);
    let main = portal.text("home-portal.toml");
    assert!(
        main.starts_with("[permissions]\nrequest_at_start = false\n"),
        "{main}"
    );
    assert!(!portal.text("services.toml").contains("permissions"));
    assert!(crate::Section::MAIN_KEYS.contains(&"permissions"));
}

#[test]
fn a_second_start_rewrites_nothing() {
    let portal = Portal::with(&[("home-portal.toml", ALL_IN_ONE)]);
    let stamp = |name: &str| fs::metadata(portal.path(name)).unwrap().modified().unwrap();
    let before = (
        stamp("home-portal.toml"),
        stamp("services.toml"),
        stamp("workflows/revive.toml"),
    );
    crate::ConfigStore::open(portal.path("home-portal.toml")).unwrap();
    assert_eq!(
        before,
        (
            stamp("home-portal.toml"),
            stamp("services.toml"),
            stamp("workflows/revive.toml")
        )
    );
}

#[test]
fn a_collision_stops_the_start_and_changes_nothing() {
    let directory = tempfile::tempdir().unwrap();
    let main = "[proxy]\nportal_host = \"a\"\n";
    let proxy = "[proxy]\nportal_host = \"b\"\n";
    fs::write(directory.path().join("home-portal.toml"), main).unwrap();
    fs::write(directory.path().join("proxy.toml"), proxy).unwrap();
    let message = crate::ConfigStore::open(directory.path().join("home-portal.toml"))
        .err()
        .unwrap()
        .to_string();
    assert!(message.contains("proxy"), "{message}");
    assert!(
        message.contains("home-portal.toml") && message.contains("proxy.toml"),
        "{message}"
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("home-portal.toml")).unwrap(),
        main
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("proxy.toml")).unwrap(),
        proxy
    );
}

#[test]
fn workflows_without_a_valid_id_or_with_one_id_twice_are_not_moved() {
    for text in [
        "[[workflows]]\ntitle = \"No id\"\n",
        "[[workflows]]\nid = \"a\"\n\n[[workflows]]\nid = \"a\"\n",
    ] {
        let message = open(&[("home-portal.toml", text)])
            .err()
            .unwrap()
            .to_string();
        assert!(message.contains("workflow"), "{message}");
    }
}

#[test]
fn sections_leave_an_included_file_and_include_is_kept() {
    let portal = Portal::with(&[
        ("home-portal.toml", "include = [\"extra.toml\"]\n"),
        (
            "extra.toml",
            "# my services\n\n[[services]]\nid = \"a\"\n\n[[dashboard.widgets]]\ntype = \"weather\"\n",
        ),
    ]);
    assert_eq!(
        portal.text("home-portal.toml"),
        "include = [\"extra.toml\"]\n"
    );
    assert!(!portal.text("extra.toml").contains("[["));
    assert!(portal.text("services.toml").contains("# my services"));
    assert!(
        portal
            .text("dashboard.toml")
            .contains("[[dashboard.widgets]]")
    );
}

#[test]
fn a_shared_home_holds_both_of_its_sections() {
    let portal = Portal::with(&[(
        "home-portal.toml",
        "[files]\nautomations = \"rules.toml\"\nwebhooks = \"rules.toml\"\n\n[[automations]]\nid = \"a\"\n\n[[webhooks]]\nid = \"w\"\n",
    )]);
    let rules = portal.text("rules.toml");
    assert!(
        rules.contains("[[automations]]") && rules.contains("[[webhooks]]"),
        "{rules}"
    );
    assert!(portal.text("home-portal.toml").starts_with("[files]"));
}

#[test]
fn the_comment_above_a_workflow_opens_its_file_and_new_homes_start_without_blank_lines() {
    let portal = Portal::with(&[("home-portal.toml", ALL_IN_ONE)]);
    assert!(
        portal
            .text("workflows/revive.toml")
            .starts_with("# revive a service\nid = \"revive\""),
        "{}",
        portal.text("workflows/revive.toml")
    );
    assert!(
        portal.text("services.toml").starts_with("# the media box"),
        "{}",
        portal.text("services.toml")
    );
    assert!(
        portal.text("proxy.toml").starts_with("[proxy]"),
        "{}",
        portal.text("proxy.toml")
    );
}

#[test]
fn groups_move_into_the_home_of_the_users() {
    let portal = Portal::with(&[(
        "home-portal.toml",
        "[network]\nport = 8080\n\n# the family\n[[groups]]\nname = \"family\"\npermissions = { automations = [\"read\"] }\n",
    )]);
    assert_eq!(
        portal.text("home-portal.toml").trim(),
        "[network]\nport = 8080"
    );
    let users = portal.text("users.toml");
    assert!(
        users.contains("# the family\n[[groups]]\nname = \"family\""),
        "{users}"
    );
}
