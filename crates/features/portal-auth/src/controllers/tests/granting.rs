use axum::http::StatusCode;

use super::support::{
    ON, Portal, entry, member, portal, revision, send, signed_in, users_text_of, write,
};

const GROUPS: &str = "[[groups]]\nname = \"family\"\npermissions = { users = [\"read\", \"create\", \"update\", \"delete\"], automations = [\"read\"] }\n\n[[groups]]\nname = \"guests\"\n\n[[groups]]\nname = \"operators\"\npermissions = { proxy = [\"update\"] }\n";

fn household() -> Portal {
    portal(format!(
        "{ON}{}\n{}\n{}\n{GROUPS}",
        entry("root", "secret99"),
        member("anna", "correct horse", Some("family")),
        member("bob", "correct horse", None)
    ))
}

async fn as_person(portal: &Portal, name: &str) -> (String, String) {
    let password = if name == "root" {
        "secret99"
    } else {
        "correct horse"
    };
    let cookie = signed_in(portal, name, password).await.unwrap();
    let root = signed_in(portal, "root", "secret99").await.unwrap();
    let revision = revision(portal, &root).await;
    (cookie, revision)
}

async fn status_of(portal: &Portal, name: &str, method: &str, uri: &str, body: &str) -> StatusCode {
    let (cookie, revision) = as_person(portal, name).await;
    send(portal, write(method, uri, &cookie, &revision, body))
        .await
        .0
}

#[tokio::test]
async fn a_user_added_in_a_group_signs_in_with_its_rights() {
    let portal = household();
    let status = status_of(
        &portal,
        "root",
        "POST",
        "/api/users",
        r#"{"name":"carol","password":"correct horse","group":"family"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(
        users_text_of(&portal).contains("name = \"carol\""),
        "{}",
        users_text_of(&portal)
    );
    assert!(users_text_of(&portal).contains("group = \"family\""));
    assert!(signed_in(&portal, "carol", "correct horse").await.is_some());
}

#[tokio::test]
async fn an_unknown_group_names_the_group_field() {
    let portal = household();
    let (cookie, revision) = as_person(&portal, "root").await;
    let (status, _, body) = send(
        &portal,
        write(
            "POST",
            "/api/users",
            &cookie,
            &revision,
            r#"{"name":"carol","password":"correct horse","group":"strangers"}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["errors"][0]["field"], "group");
}

#[tokio::test]
async fn a_guest_changes_their_own_password_but_no_one_elses() {
    let portal = household();
    let other = status_of(
        &portal,
        "bob",
        "PUT",
        "/api/users/anna/password",
        r#"{"password":"a brand new one"}"#,
    )
    .await;
    assert_eq!(other, StatusCode::FORBIDDEN);
    let own = status_of(
        &portal,
        "bob",
        "PUT",
        "/api/users/bob/password",
        r#"{"password":"a brand new one"}"#,
    )
    .await;
    assert_eq!(own, StatusCode::OK);
    assert!(signed_in(&portal, "bob", "a brand new one").await.is_some());
}

#[tokio::test]
async fn moving_anna_to_guests_writes_her_group() {
    let portal = household();
    let status = status_of(
        &portal,
        "root",
        "PUT",
        "/api/users/anna/group",
        r#"{"group":"guests"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let text = users_text_of(&portal);
    assert!(
        text.contains("name = \"anna\"") && text.contains("group = \"guests\""),
        "{text}"
    );
    assert!(!text.contains("group = \"family\"\n\n[[users]]\nname = \"bob\""));
}

#[tokio::test]
async fn the_last_admin_cannot_leave_admin() {
    let portal = household();
    let before = users_text_of(&portal);
    let status = status_of(
        &portal,
        "root",
        "PUT",
        "/api/users/root/group",
        r#"{"group":"family"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(users_text_of(&portal), before);
}

#[tokio::test]
async fn promoting_oneself_is_refused() {
    let portal = household();
    let status = status_of(
        &portal,
        "anna",
        "PUT",
        "/api/users/anna/group",
        r#"{"group":"admin"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(users_text_of(&portal).contains("group = \"family\""));
}

#[tokio::test]
async fn giving_a_group_with_more_rights_is_refused_and_a_smaller_one_is_not() {
    let portal = household();
    let wider = status_of(
        &portal,
        "anna",
        "POST",
        "/api/users",
        r#"{"name":"carol","password":"correct horse","group":"operators"}"#,
    )
    .await;
    assert_eq!(wider, StatusCode::FORBIDDEN);
    assert!(!users_text_of(&portal).contains("carol"));
    let smaller = status_of(
        &portal,
        "anna",
        "PUT",
        "/api/users/bob/group",
        r#"{"group":"guests"}"#,
    )
    .await;
    assert_eq!(smaller, StatusCode::OK);
}

#[tokio::test]
async fn taking_over_an_admin_is_refused() {
    let portal = household();
    let password = status_of(
        &portal,
        "anna",
        "PUT",
        "/api/users/root/password",
        r#"{"password":"a brand new one"}"#,
    )
    .await;
    assert_eq!(password, StatusCode::FORBIDDEN);
    let deleted = status_of(&portal, "anna", "DELETE", "/api/users/root", "").await;
    assert_eq!(deleted, StatusCode::FORBIDDEN);
    assert!(signed_in(&portal, "root", "secret99").await.is_some());
}
