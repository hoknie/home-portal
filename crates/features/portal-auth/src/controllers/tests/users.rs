use std::fs;
use std::sync::Arc;

use axum::http::StatusCode;
use portal_config::{ConfigStore, Revision};
use portal_feature::ApiError;

use super::support::{
    OFF, ON, entry, get, portal, portal_with, revision, send, signed_in, text_of, write,
};
use crate::features::AuthFeature;
use crate::usecases::DeleteUser;

const NEW: &str = r#"{"name":"anna","password":"correct horse"}"#;

#[tokio::test]
async fn a_user_added_from_the_interface_signs_in_with_that_password() {
    let portal = portal(format!("# people\n{ON}{}", entry("admin", "secret99")));
    let cookie = signed_in(&portal, "admin", "secret99").await.unwrap();
    let revision = revision(&portal, &cookie).await;
    let (status, _, body) = send(
        &portal,
        write("POST", AuthFeature::USERS, &cookie, &revision, NEW),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["users"][1]["name"], "anna");
    let text = text_of(&portal);
    assert!(text.starts_with("# people\n"), "{text}");
    assert!(
        text.contains("name = \"anna\"\npassword_hash = \"$argon2id$"),
        "{text}"
    );
    assert!(!text.contains("correct horse"));
    assert!(signed_in(&portal, "anna", "correct horse").await.is_some());
}

#[tokio::test]
async fn a_taken_name_and_a_short_password_are_refused_by_field_and_leave_the_file() {
    let portal = portal(format!("{ON}{}", entry("admin", "secret99")));
    let cookie = signed_in(&portal, "admin", "secret99").await.unwrap();
    let before = text_of(&portal);
    let revision = revision(&portal, &cookie).await;
    let body = r#"{"name":" admin ","password":"short"}"#;
    let (status, _, answer) = send(
        &portal,
        write("POST", AuthFeature::USERS, &cookie, &revision, body),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let fields: Vec<&str> = answer["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|error| error["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, vec!["name", "password"]);
    assert!(!answer.to_string().contains("short"));
    assert_eq!(text_of(&portal), before);
}

#[tokio::test]
async fn a_user_is_added_to_the_included_file_that_holds_the_users() {
    let portal = portal_with(&[
        (
            "home-portal.toml",
            format!("include = [\"people.toml\"]\n\n{ON}"),
        ),
        ("people.toml", entry("admin", "secret99")),
    ]);
    let cookie = signed_in(&portal, "admin", "secret99").await.unwrap();
    let main = text_of(&portal);
    let revision = revision(&portal, &cookie).await;
    let (status, _, _) = send(
        &portal,
        write("POST", AuthFeature::USERS, &cookie, &revision, NEW),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(text_of(&portal), main);
    let people = fs::read_to_string(portal.folder.path().join("people.toml")).unwrap();
    assert!(people.contains("name = \"anna\""), "{people}");
}

#[tokio::test]
async fn the_list_marks_you_and_holds_no_hash() {
    let portal = portal(format!(
        "{}\n{}",
        entry("admin", "secret99"),
        entry("anna", "correct horse")
    ));
    let cookie = signed_in(&portal, "admin", "secret99").await.unwrap();
    let (status, etag, body) = send(&portal, get(AuthFeature::USERS, &cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(etag.is_some());
    assert_eq!(body["users"][0]["name"], "admin");
    assert_eq!(body["users"][0]["you"], true);
    assert_eq!(body["users"][1]["you"], false);
    assert_eq!(body["editable"], false);
    assert!(!body.to_string().contains("$argon2id$"));
}

#[tokio::test]
async fn writes_while_the_module_is_off_are_refused_and_signing_in_still_works() {
    let portal = portal(format!(
        "{OFF}{}\n{}",
        entry("admin", "secret99"),
        entry("anna", "correct horse")
    ));
    let cookie = signed_in(&portal, "admin", "secret99").await.unwrap();
    let before = text_of(&portal);
    let revision = revision(&portal, &cookie).await;
    for (method, uri, body) in [
        ("POST", AuthFeature::USERS.to_string(), NEW),
        (
            "PUT",
            "/api/users/anna/password".to_string(),
            r#"{"password":"a brand new one"}"#,
        ),
        ("DELETE", "/api/users/anna".to_string(), ""),
    ] {
        let (status, _, answer) =
            send(&portal, write(method, &uri, &cookie, &revision, body)).await;
        assert_eq!(status, StatusCode::CONFLICT, "{method} {uri}");
        assert!(
            answer.as_str().unwrap().contains("users module is off"),
            "{answer}"
        );
    }
    assert_eq!(text_of(&portal), before);
    assert!(signed_in(&portal, "anna", "correct horse").await.is_some());
}

#[tokio::test]
async fn deleting_yourself_is_refused_and_an_unknown_user_is_not_found() {
    let portal = portal(format!(
        "{ON}{}\n{}",
        entry("admin", "secret99"),
        entry("anna", "correct horse")
    ));
    let cookie = signed_in(&portal, "admin", "secret99").await.unwrap();
    let revision = revision(&portal, &cookie).await;
    let (status, _, answer) = send(
        &portal,
        write("DELETE", "/api/users/admin", &cookie, &revision, ""),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(answer.as_str().unwrap().contains("yourself"));
    let (missing, _, _) = send(
        &portal,
        write("DELETE", "/api/users/nobody", &cookie, &revision, ""),
    )
    .await;
    assert_eq!(missing, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_last_user_cannot_be_deleted() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("home-portal.toml");
    fs::write(&path, format!("{ON}{}", entry("admin", "secret99"))).unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let revision: Revision = store.read().revision;
    let refused = DeleteUser::new(store)
        .run("someone-else", "admin", &revision)
        .await;
    assert!(
        matches!(refused, Err(ApiError::Conflict(message)) if message.contains("at least one user"))
    );
}

#[tokio::test]
async fn deleting_another_user_ends_their_session() {
    let portal = portal(format!(
        "{ON}{}\n# my sister\n{}",
        entry("admin", "secret99"),
        entry("anna", "correct horse")
    ));
    let admin = signed_in(&portal, "admin", "secret99").await.unwrap();
    let anna = signed_in(&portal, "anna", "correct horse").await.unwrap();
    let revision = revision(&portal, &admin).await;
    let (status, _, body) = send(
        &portal,
        write("DELETE", "/api/users/anna", &admin, &revision, ""),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["users"].as_array().unwrap().len(), 1);
    assert!(!text_of(&portal).contains("anna"));
    let (after, _, _) = send(&portal, get(AuthFeature::USERS, &anna)).await;
    assert_eq!(after, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_stale_revision_is_refused() {
    let portal = portal(format!("{ON}{}", entry("admin", "secret99")));
    let cookie = signed_in(&portal, "admin", "secret99").await.unwrap();
    let (status, _, _) = send(
        &portal,
        write("POST", AuthFeature::USERS, &cookie, "\"old\"", NEW),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}
