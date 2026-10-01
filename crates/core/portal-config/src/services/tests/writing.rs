use portal_feature::ApiError;
use toml_edit::value;

use super::support::{Portal, rewrite, services_have_names};
use crate::types::Section;

#[tokio::test]
async fn a_write_lands_in_the_file_that_holds_the_entry() {
    let portal = Portal::with(&[
        ("home-portal.toml", "[network]\nport = 8080\n"),
        (
            "services.toml",
            "# other services\n\n[[services]]\nid = \"nas\"\nname = \"NAS\"\n",
        ),
    ]);
    let main_before = portal.text("home-portal.toml");
    let revision = portal.store.read().revision;
    portal
        .store
        .update(&portal.path("services.toml"), &revision, |document| {
            document["services"][0]["name"] = value("Storage");
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(portal.text("home-portal.toml"), main_before);
    let included = portal.text("services.toml");
    assert!(included.contains("# other services"), "{included}");
    assert!(included.contains("name = \"Storage\""), "{included}");
}

#[tokio::test]
async fn a_new_entry_goes_to_its_home() {
    let portal = Portal::with(&[
        ("home-portal.toml", "[network]\nport = 8080\n"),
        ("services.toml", "[[services]]\nid = \"a\"\n"),
    ]);
    let main_before = portal.text("home-portal.toml");
    let revision = portal.store.read().revision;
    let target = portal.store.home_of(Section::Services);
    assert_eq!(target, portal.path("services.toml"));
    portal
        .store
        .update(&target, &revision, |document| {
            document["services"]
                .as_array_of_tables_mut()
                .unwrap()
                .push(toml_edit::Table::new());
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(portal.text("home-portal.toml"), main_before);
    assert_eq!(
        portal.text("services.toml").matches("[[services]]").count(),
        2
    );
}

#[tokio::test]
async fn the_first_entry_creates_its_home_with_mode_0600() {
    let portal = Portal::with(&[("home-portal.toml", "[network]\nport = 8080\n")]);
    let revision = portal.store.read().revision;
    let target = portal.store.home_of(Section::Webhooks);
    portal
        .store
        .update(&target, &revision, |document| {
            let mut entries = toml_edit::ArrayOfTables::new();
            let mut entry = toml_edit::Table::new();
            entry["id"] = value("deploy");
            entries.push(entry);
            document.insert("webhooks", toml_edit::Item::ArrayOfTables(entries));
            Ok(())
        })
        .await
        .unwrap();
    assert!(portal.text("webhooks.toml").contains("[[webhooks]]"));
    assert_eq!(mode(&portal.path("webhooks.toml")), 0o600);
    assert!(!portal.path("webhooks.toml.previous").exists());
    assert_eq!(
        portal.store.read().document["webhooks"][0]["id"].as_str(),
        Some("deploy")
    );
}

#[tokio::test]
async fn a_workflow_is_created_renamed_and_removed_as_its_own_file() {
    let portal = Portal::with(&[("home-portal.toml", "[network]\nport = 8080\n")]);
    let file = portal.store.workflow_file("revive");
    let revision = portal.store.read().revision;
    let (_, snapshot) = portal
        .store
        .update(&file, &revision, |document| {
            let mut entry = toml_edit::Table::new();
            entry["id"] = value("revive");
            entry["title"] = value("Revive");
            document["workflows"]
                .as_array_of_tables_mut()
                .unwrap()
                .push(entry);
            Ok(())
        })
        .await
        .unwrap();
    let text = portal.text("workflows/revive.toml");
    assert!(text.starts_with("id = \"revive\""), "{text}");
    assert!(!text.contains("[[workflows"), "{text}");
    assert_eq!(mode(&portal.path("workflows")), 0o700);
    assert_eq!(mode(&file), 0o600);
    let renamed = portal.store.workflow_file("bring-back");
    let (_, snapshot) = portal
        .store
        .update_moved(&file, &renamed, &snapshot.revision, |document| {
            document["workflows"][0]["id"] = value("bring-back");
            Ok(())
        })
        .await
        .unwrap();
    assert!(!file.exists());
    assert!(
        portal
            .text("workflows/bring-back.toml")
            .contains("bring-back")
    );
    assert!(
        portal
            .text("workflows/revive.toml.previous")
            .contains("revive")
    );
    assert_eq!(
        snapshot.document["workflows"][0]["id"].as_str(),
        Some("bring-back")
    );
    let snapshot = portal
        .store
        .remove(&renamed, &snapshot.revision)
        .await
        .unwrap();
    assert!(!renamed.exists());
    assert!(snapshot.document.get("workflows").is_none());
}

#[tokio::test]
async fn a_workflow_file_dropped_in_by_hand_is_read_and_makes_an_older_revision_stale() {
    let portal = Portal::with(&[("home-portal.toml", "[network]\nport = 8080\n")]);
    let revision = portal.store.read().revision;
    std::fs::create_dir(portal.path("workflows")).unwrap();
    std::fs::write(
        portal.path("workflows/revive.toml"),
        "id = \"revive\"\ntitle = \"Revive\"\n",
    )
    .unwrap();
    let snapshot = portal.store.read();
    assert_eq!(
        snapshot.document["workflows"][0]["id"].as_str(),
        Some("revive")
    );
    let refused = portal
        .store
        .update(&portal.store.home_of(Section::Services), &revision, |_| {
            Ok(())
        })
        .await;
    assert!(matches!(refused, Err(ApiError::Conflict(_))), "{refused:?}");
    std::fs::remove_file(portal.path("workflows/revive.toml")).unwrap();
    assert!(portal.store.read().document.get("workflows").is_none());
}

#[tokio::test]
async fn writes_never_touch_files() {
    let main = "[files]\nworkflows = \"flows/\"\nservices = \"lists/services.toml\"\n\n[network]\nport = 8080\n";
    let portal = Portal::with(&[("home-portal.toml", main)]);
    let revision = portal.store.read().revision;
    let (_, snapshot) = portal
        .store
        .update(
            &portal.store.home_of(Section::Services),
            &revision,
            |document| {
                let mut entries = toml_edit::ArrayOfTables::new();
                entries.push(toml_edit::Table::new());
                document.insert("services", toml_edit::Item::ArrayOfTables(entries));
                Ok(())
            },
        )
        .await
        .unwrap();
    let flow = portal.store.workflow_file("nightly");
    assert_eq!(flow, portal.path("flows/nightly.toml"));
    portal
        .store
        .update(&flow, &snapshot.revision, |document| {
            let mut entry = toml_edit::Table::new();
            entry["id"] = value("nightly");
            document["workflows"]
                .as_array_of_tables_mut()
                .unwrap()
                .push(entry);
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(portal.text("home-portal.toml"), main);
    assert!(portal.path("lists/services.toml").exists());
    assert!(!portal.path("workflows").exists());
}

fn mode(path: &std::path::Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[tokio::test]
async fn an_edit_to_any_file_makes_an_older_revision_stale() {
    let portal = Portal::with(&[
        ("home-portal.toml", ""),
        ("services.toml", "[[services]]\nid = \"a\"\n"),
        (
            "dashboard.toml",
            "[[dashboard.widgets]]\ntype = \"services\"\n",
        ),
    ]);
    let revision = portal.store.read().revision;
    rewrite(
        &portal.path("dashboard.toml"),
        "[[dashboard.widgets]]\ntype = \"status-summary\"\n",
    );
    let refused = portal
        .store
        .update(&portal.path("services.toml"), &revision, |document| {
            document["services"][0]["id"] = value("b");
            Ok(())
        })
        .await;
    assert!(matches!(refused, Err(ApiError::Conflict(_))), "{refused:?}");
}

#[tokio::test]
async fn a_write_that_breaks_a_rule_is_refused_field_by_field() {
    let portal = Portal::with(&[
        ("home-portal.toml", ""),
        ("services.toml", "[[services]]\nid = \"a\"\nname = \"A\"\n"),
    ]);
    portal.store.adopt(vec![services_have_names]).unwrap();
    let revision = portal.store.read().revision;
    let refused = portal
        .store
        .update(&portal.path("services.toml"), &revision, |document| {
            document["services"][0]
                .as_table_mut()
                .unwrap()
                .remove("name");
            Ok(())
        })
        .await;
    match refused {
        Err(ApiError::Invalid(errors)) => assert_eq!(errors[0].field, "services[0].name"),
        other => panic!("expected Invalid, got {other:?}"),
    }
}
