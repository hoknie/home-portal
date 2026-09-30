use axum::http::StatusCode;
use portal_feature::{Action, Area, Feature, Principal, Right, Rights};

use crate::controllers::tests::automations::{Api, api, revision, send, write};
use crate::features::AutomationsFeature;

const FILE: &str = "[modules]\nworkflows = true\n";

fn as_member(rights: Vec<Right>) -> Api {
    let mut api = api(FILE);
    api.router = api
        .feature
        .router()
        .layer(axum::Extension(Principal::member(
            "anna",
            Some("family".to_string()),
            Rights::of(rights),
        )))
        .merge(api.feature.public_router());
    api
}

fn workflow(header: &str) -> String {
    format!(
        r#"{{"id":"peek","title":"Peek","steps":[{{"id":"call","kind":"http","url":"https://example.net/","headers":{{"Authorization":"{header}"}}}}]}}"#
    )
}

const WORKFLOWS: [Right; 2] = [
    Right::new(Area::Workflows, Action::Create),
    Right::new(Area::Workflows, Action::Update),
];

#[tokio::test]
async fn a_family_member_cannot_save_a_workflow_that_names_a_secret() {
    let api = as_member(WORKFLOWS.to_vec());
    let current = revision(&api).await;
    let (status, _, body) = send(
        &api,
        write(
            "POST",
            AutomationsFeature::WORKFLOWS,
            Some(&current),
            &workflow("Bearer {{secrets.nas_token}}"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert!(
        body.to_string().contains("steps[0].headers.Authorization"),
        "{body}"
    );
    let (status, _, _) = send(
        &api,
        write(
            "POST",
            AutomationsFeature::WORKFLOWS,
            Some(&current),
            &workflow("Bearer none"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn a_person_who_reads_secrets_may_name_one() {
    let mut rights = WORKFLOWS.to_vec();
    rights.push(Right::new(Area::Secrets, Action::Read));
    let api = as_member(rights);
    let current = revision(&api).await;
    let (status, _, body) = send(
        &api,
        write(
            "POST",
            AutomationsFeature::WORKFLOWS,
            Some(&current),
            &workflow("Bearer {{secrets.nas_token}}"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
}
