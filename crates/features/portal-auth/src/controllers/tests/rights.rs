use std::fs;

use portal_feature::{Action, Area, Right, Rights};

use axum::http::StatusCode;
use serde_json::json;

use super::support::{
    ON, cookie_headers, entry, get, member, portal, send, signed_in, users_text_of,
};
use crate::features::AuthFeature;

const FAMILY: &str = "[[groups]]\nname = \"family\"\npermissions = { automations = [\"read\", \"execute\"] }\n\n[[groups]]\nname = \"guests\"\n";

fn family_rights() -> Rights {
    Rights::of([
        Right::new(Area::Automations, Action::Read),
        Right::new(Area::Automations, Action::Execute),
    ])
}

#[tokio::test]
async fn each_request_carries_the_rights_of_the_current_group() {
    let portal = portal(format!(
        "{ON}{}\n{}\n{}\n{FAMILY}",
        entry("root", "secret99"),
        member("anna", "correct horse", Some("family")),
        member("guest", "correct horse", None)
    ));
    let root = signed_in(&portal, "root", "secret99").await.unwrap();
    let anna = signed_in(&portal, "anna", "correct horse").await.unwrap();
    let guest = signed_in(&portal, "guest", "correct horse").await.unwrap();
    let admitted = |cookie: &str| portal.gate.admit(&cookie_headers(cookie)).unwrap();
    assert!(admitted(&root).rights.is_admin());
    assert_eq!(admitted(&anna).group.as_deref(), Some("family"));
    assert_eq!(admitted(&anna).rights, family_rights());
    assert_eq!(admitted(&guest).group, None);
    assert_eq!(admitted(&guest).rights, Rights::none());
    let moved = users_text_of(&portal).replace("group = \"family\"", "group = \"guests\"");
    fs::write(portal.folder.path().join("users.toml"), moved).unwrap();
    let anna_now = admitted(&anna);
    assert_eq!(anna_now.group.as_deref(), Some("guests"));
    assert_eq!(anna_now.rights, Rights::none());
}

#[tokio::test]
async fn the_session_answers_the_group_and_its_rights() {
    let portal = portal(format!(
        "{ON}{}\n{}\n{FAMILY}",
        entry("root", "secret99"),
        member("anna", "correct horse", Some("family"))
    ));
    let anna = signed_in(&portal, "anna", "correct horse").await.unwrap();
    let (status, _, body) = send(&portal, get(AuthFeature::PATH, &anna)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        json!({"name": "anna", "group": "family", "admin": false, "rights": {"automations": ["read", "execute"]}})
    );
}
