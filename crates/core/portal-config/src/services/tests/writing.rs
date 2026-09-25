use portal_feature::ApiError;
use toml_edit::value;

use super::support::{COMMENTED, Portal, open, rewrite, services_have_names};

#[tokio::test]
async fn a_write_lands_in_the_file_that_holds_the_entry() {
    let portal = Portal::with(&[
        (
            "home-portal.toml",
            format!("include = [\"services.toml\"]\n\n{COMMENTED}").as_str(),
        ),
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
async fn a_new_entry_lands_in_the_file_configuration_names() {
    let portal = Portal::with(&[
        (
            "home-portal.toml",
            "include = [\"services.toml\"]\n\n[configuration]\nwrites_to = \"services.toml\"\n",
        ),
        ("services.toml", "[[services]]\nid = \"a\"\n"),
    ]);
    assert_eq!(portal.store.writes_to(), portal.path("services.toml"));
    let revision = portal.store.read().revision;
    let target = portal.store.writes_to();
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
    assert_eq!(
        portal
            .text("home-portal.toml")
            .matches("[[services]]")
            .count(),
        0
    );
    assert_eq!(
        portal.text("services.toml").matches("[[services]]").count(),
        2
    );
}

#[test]
fn a_writes_to_that_names_no_configuration_file_is_refused() {
    let error = open(&[(
        "home-portal.toml",
        "[configuration]\nwrites_to = \"elsewhere.toml\"\n",
    )])
    .err()
    .unwrap();
    assert!(error.to_string().contains("writes_to"), "{error}");
}

#[tokio::test]
async fn an_edit_to_any_file_makes_an_older_revision_stale() {
    let portal = Portal::with(&[
        (
            "home-portal.toml",
            "include = [\"widgets.toml\"]\n\n[[services]]\nid = \"a\"\n",
        ),
        (
            "widgets.toml",
            "[[dashboard.widgets]]\ntype = \"services\"\n",
        ),
    ]);
    let revision = portal.store.read().revision;
    rewrite(
        &portal.path("widgets.toml"),
        "[[dashboard.widgets]]\ntype = \"status-summary\"\n",
    );
    let refused = portal
        .store
        .update(&portal.path("home-portal.toml"), &revision, |document| {
            document["services"][0]["id"] = value("b");
            Ok(())
        })
        .await;
    assert!(matches!(refused, Err(ApiError::Conflict(_))), "{refused:?}");
}

#[tokio::test]
async fn a_write_that_breaks_a_rule_is_refused_field_by_field() {
    let portal = Portal::with(&[(
        "home-portal.toml",
        "[[services]]\nid = \"a\"\nname = \"A\"\n",
    )]);
    portal.store.adopt(vec![services_have_names]).unwrap();
    let revision = portal.store.read().revision;
    let refused = portal
        .store
        .update(&portal.path("home-portal.toml"), &revision, |document| {
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
