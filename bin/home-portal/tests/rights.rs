mod support;

use axum::Router;
use axum::body::Body;
use axum::http::header::{CONTENT_TYPE, COOKIE, SET_COOKIE};
use axum::http::{Method, Request, StatusCode};
use home_portal::{adopt, assemble, registered, rule_book};
use portal_auth::hash_password;
use portal_feature::Requirement;
use tower::ServiceExt;

const GUEST: &str = "guest";
const SIGN_OUT: (&str, &str) = ("DELETE", "/api/session");

struct Portal {
    router: Router,
    rules: Vec<(Method, &'static str, Requirement)>,
    _directory: tempfile::TempDir,
}

fn portal() -> Portal {
    let directory = tempfile::tempdir().unwrap();
    let hash = hash_password("secret").unwrap();
    let guest = format!("\n[[users]]\nname = \"{GUEST}\"\npassword_hash = \"{hash}\"\n");
    let path = support::with_extra(&directory, "secret", &guest);
    let wiring = support::wiring_for(&path);
    let registry = registered(&wiring).unwrap();
    adopt(&wiring.configuration, &registry).unwrap();
    Portal {
        router: assemble(&registry),
        rules: rule_book(&registry).rules(),
        _directory: directory,
    }
}

async fn signed_in(portal: &Portal, name: &str) -> String {
    let response = portal
        .router
        .clone()
        .oneshot(
            Request::post("/api/session")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(format!(
                    r#"{{"name":"{name}","password":"secret"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    response.headers()[SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string()
}

fn concrete(path: &str, name: &str) -> String {
    let mut out = String::new();
    let mut rest = path;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('}')
            .map_or(rest.len(), |end| start + end + 1);
        out.push_str(if &rest[start..end] == "{name}" {
            name
        } else {
            "x"
        });
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

async fn status(portal: &Portal, method: &Method, path: &str, cookie: &str) -> StatusCode {
    let mut builder = Request::builder()
        .method(method.clone())
        .uri(path)
        .header(COOKIE, cookie);
    let body = if method == Method::GET {
        Body::empty()
    } else {
        builder = builder.header(CONTENT_TYPE, "application/json");
        Body::from("{}")
    };
    portal
        .router
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn a_user_without_a_group_reaches_only_what_everyone_may_read() {
    let portal = portal();
    let cookie = signed_in(&portal, GUEST).await;
    let mut wrong = Vec::new();
    for (method, path, requirement) in &portal.rules {
        if (method.as_str(), *path) == SIGN_OUT {
            continue;
        }
        let answer = status(&portal, method, &concrete(path, GUEST), &cookie).await;
        let open = requirement == &Requirement::Signed;
        if open == (answer == StatusCode::FORBIDDEN) || answer == StatusCode::UNAUTHORIZED {
            wrong.push(format!("{method} {path}: {answer}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[tokio::test]
async fn admin_is_never_refused_a_read() {
    let portal = portal();
    let cookie = signed_in(&portal, "admin").await;
    let mut wrong = Vec::new();
    for (method, path, _) in portal
        .rules
        .iter()
        .filter(|(method, ..)| method == Method::GET)
    {
        let answer = status(&portal, method, &concrete(path, "admin"), &cookie).await;
        if matches!(answer, StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED) {
            wrong.push(format!("{method} {path}: {answer}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[tokio::test]
async fn a_guest_sees_the_status_but_not_the_network() {
    let portal = portal();
    let cookie = signed_in(&portal, GUEST).await;
    assert_eq!(
        status(&portal, &Method::GET, "/api/services", &cookie).await,
        StatusCode::OK
    );
    assert_eq!(
        status(&portal, &Method::PUT, "/api/network", &cookie).await,
        StatusCode::FORBIDDEN
    );
}
