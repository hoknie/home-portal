use std::fs;

use axum::http::StatusCode;
use serde_json::json;

use super::support::{
    OFF, ON, Portal, entry, get, member, portal, revision, send, signed_in, text_of, users_text_of,
    write,
};
use crate::features::AuthFeature;

const FAMILY: &str = "[[groups]]\nname = \"family\"\npermissions = { users = [\"read\", \"create\", \"update\", \"delete\"] }\n";

fn household(switch: &str) -> Portal {
    portal(format!(
        "{switch}{}\n{}\n{FAMILY}",
        entry("root", "secret99"),
        member("anna", "correct horse", Some("family"))
    ))
}

async fn by(
    portal: &Portal,
    name: &str,
    method: &str,
    uri: &str,
    body: &str,
) -> (StatusCode, serde_json::Value) {
    let password = if name == "root" {
        "secret99"
    } else {
        "correct horse"
    };
    let cookie = signed_in(portal, name, password).await.unwrap();
    let revision = revision(portal, &cookie).await;
    let (status, _, answer) = send(portal, write(method, uri, &cookie, &revision, body)).await;
    (status, answer)
}

#[tokio::test]
async fn the_list_puts_admin_first_with_its_members_and_answers_the_matrix() {
    let portal = household(ON);
    let cookie = signed_in(&portal, "root", "secret99").await.unwrap();
    let (status, etag, body) = send(&portal, get(AuthFeature::GROUPS, &cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(etag.is_some());
    assert_eq!(body["groups"][0]["name"], "admin");
    assert_eq!(body["groups"][0]["builtin"], true);
    assert_eq!(body["groups"][0]["members"], json!(["root"]));
    assert_eq!(
        body["groups"][0]["rights"]["workflows"],
        json!(["read", "create", "update", "delete", "execute"])
    );
    assert_eq!(body["groups"][1]["name"], "family");
    assert_eq!(body["groups"][1]["members"], json!(["anna"]));
    assert_eq!(
        body["matrix"][0],
        json!({"area": "services", "actions": ["create", "update", "delete"]})
    );
}

#[tokio::test]
async fn creating_a_group_writes_it_with_its_permissions() {
    let portal = household(ON);
    let (status, body) = by(
        &portal,
        "root",
        "POST",
        AuthFeature::GROUPS,
        r#"{"name":"guests","rights":{"services":["update"],"automations":["execute","read"]}}"#,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["groups"][2]["name"], "guests");
    assert_eq!(
        body["groups"][2]["rights"]["automations"],
        json!(["read", "execute"])
    );
    let text = users_text_of(&portal);
    assert!(
        text.contains("name = \"guests\"\npermissions = { services = [\"update\"], automations = [\"read\", \"execute\"] }"),
        "{text}"
    );
}

#[tokio::test]
async fn a_bad_name_or_right_names_its_field() {
    let portal = household(ON);
    let (status, body) = by(
        &portal,
        "root",
        "POST",
        AuthFeature::GROUPS,
        r#"{"name":"admin","rights":{"layout":["delete"]}}"#,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let fields: Vec<&str> = body["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|error| error["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, vec!["name", "rights.layout"]);
}

#[tokio::test]
async fn a_group_with_members_is_not_deleted() {
    let portal = household(ON);
    let before = users_text_of(&portal);
    let (status, body) = by(&portal, "root", "DELETE", "/api/groups/family", "").await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body.as_str().unwrap().contains("anna"), "{body}");
    assert_eq!(users_text_of(&portal), before);
}

#[tokio::test]
async fn an_empty_group_is_deleted_and_admin_never_is() {
    let portal = portal(format!(
        "{ON}{}\n[[groups]]\nname = \"empty\"\n",
        entry("root", "secret99")
    ));
    let (admin, _) = by(&portal, "root", "DELETE", "/api/groups/admin", "").await;
    assert_eq!(admin, StatusCode::CONFLICT);
    let (unknown, _) = by(&portal, "root", "DELETE", "/api/groups/strangers", "").await;
    assert_eq!(unknown, StatusCode::NOT_FOUND);
    let (status, body) = by(&portal, "root", "DELETE", "/api/groups/empty", "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["groups"].as_array().unwrap().len(), 1);
    assert!(!users_text_of(&portal).contains("empty"));
}

#[tokio::test]
async fn a_member_of_another_group_may_not_write_groups() {
    let portal = household(ON);
    let (status, _) = by(
        &portal,
        "anna",
        "POST",
        AuthFeature::GROUPS,
        r#"{"name":"guests"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(!users_text_of(&portal).contains("guests"));
}

#[tokio::test]
async fn writes_need_the_users_module() {
    let portal = household(OFF);
    let (status, _) = by(
        &portal,
        "root",
        "POST",
        AuthFeature::GROUPS,
        r#"{"name":"guests"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn renaming_a_group_renames_every_member() {
    let portal = household(ON);
    let (status, body) = by(
        &portal,
        "root",
        "PUT",
        "/api/groups/family",
        r#"{"name":"household","rights":{"users":["read"]}}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["groups"][1]["name"], "household");
    assert_eq!(body["groups"][1]["members"], json!(["anna"]));
    let text = users_text_of(&portal);
    assert!(text.contains("group = \"household\""), "{text}");
    assert!(!text.contains("family"), "{text}");
}

#[tokio::test]
async fn renaming_follows_members_kept_in_another_file() {
    let portal = household(ON);
    let cookie = signed_in(&portal, "root", "secret99").await.unwrap();
    let main = format!(
        "{}\n{}",
        text_of(&portal),
        member("bob", "correct horse", Some("family"))
    );
    fs::write(&portal.main, main).unwrap();
    let revision = revision(&portal, &cookie).await;
    let (status, _, body) = send(
        &portal,
        write(
            "PUT",
            "/api/groups/family",
            &cookie,
            &revision,
            r#"{"name":"household","rights":{}}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["groups"][1]["members"], json!(["anna", "bob"]));
    assert!(text_of(&portal).contains("group = \"household\""));
    assert!(!users_text_of(&portal).contains("family"));
}
