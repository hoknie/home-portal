use axum::http::StatusCode;

use super::support::{ON, entry, get, portal, revision, send, signed_in, users_text_of, write};
use crate::features::AuthFeature;

const NEW_PASSWORD: &str = r#"{"password":"a brand new one"}"#;

#[tokio::test]
async fn resetting_another_users_password_ends_their_session_and_replaces_it() {
    let portal = portal(format!(
        "{ON}{}\n{}",
        entry("admin", "secret99"),
        entry("anna", "correct horse")
    ));
    let admin = signed_in(&portal, "admin", "secret99").await.unwrap();
    let anna = signed_in(&portal, "anna", "correct horse").await.unwrap();
    let revision = revision(&portal, &admin).await;
    let (status, _, _) = send(
        &portal,
        write(
            "PUT",
            "/api/users/anna/password",
            &admin,
            &revision,
            NEW_PASSWORD,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(!users_text_of(&portal).contains("a brand new one"));
    let (after, _, _) = send(&portal, get(AuthFeature::USERS, &anna)).await;
    assert_eq!(after, StatusCode::UNAUTHORIZED);
    assert!(signed_in(&portal, "anna", "correct horse").await.is_none());
    assert!(
        signed_in(&portal, "anna", "a brand new one")
            .await
            .is_some()
    );
    let (kept, _, _) = send(&portal, get(AuthFeature::USERS, &admin)).await;
    assert_eq!(kept, StatusCode::OK);
}

#[tokio::test]
async fn changing_your_own_password_keeps_this_session_and_ends_the_other() {
    let portal = portal(format!("{ON}{}", entry("admin", "secret99")));
    let laptop = signed_in(&portal, "admin", "secret99").await.unwrap();
    let phone = signed_in(&portal, "admin", "secret99").await.unwrap();
    let revision = revision(&portal, &laptop).await;
    let (status, _, _) = send(
        &portal,
        write(
            "PUT",
            "/api/users/admin/password",
            &laptop,
            &revision,
            NEW_PASSWORD,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (kept, _, _) = send(&portal, get(AuthFeature::USERS, &laptop)).await;
    assert_eq!(kept, StatusCode::OK);
    let (ended, _, _) = send(&portal, get(AuthFeature::USERS, &phone)).await;
    assert_eq!(ended, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_short_password_or_an_unknown_user_is_refused() {
    let portal = portal(format!("{ON}{}", entry("admin", "secret99")));
    let cookie = signed_in(&portal, "admin", "secret99").await.unwrap();
    let before = users_text_of(&portal);
    let revision = revision(&portal, &cookie).await;
    let (short, _, answer) = send(
        &portal,
        write(
            "PUT",
            "/api/users/admin/password",
            &cookie,
            &revision,
            r#"{"password":"1234567"}"#,
        ),
    )
    .await;
    assert_eq!(short, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(answer["errors"][0]["field"], "password");
    let (unknown, _, _) = send(
        &portal,
        write(
            "PUT",
            "/api/users/nobody/password",
            &cookie,
            &revision,
            NEW_PASSWORD,
        ),
    )
    .await;
    assert_eq!(unknown, StatusCode::NOT_FOUND);
    assert_eq!(users_text_of(&portal), before);
}
