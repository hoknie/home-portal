use axum::http::StatusCode;
use portal_feature::{Action, Area, Principal, Right, Rights, WidgetProvider};
use serde_json::{Value, json};

use super::support::{data_of, send, settings_of, widgets_api};

#[tokio::test]
async fn pressing_a_button_starts_its_automation_or_workflow_for_the_person_with_rendered_values() {
    let api = widgets_api(Principal::admin("anna"));
    data_of(&api, "disks").await.unwrap();
    let (status, body) = send(&api, "POST", "/api/widgets/disks/actions/restart", "").await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    assert_eq!(body["run_id"], "1");
    let (again, _) = send(&api, "POST", "/api/widgets/disks/actions/restart", "").await;
    assert_eq!(again, StatusCode::TOO_MANY_REQUESTS);
    let (workflow, _) = send(&api, "POST", "/api/widgets/disks/actions/again", "").await;
    assert_eq!(workflow, StatusCode::ACCEPTED);
    assert_eq!(
        *api.runs.started.lock().unwrap(),
        [
            r#"disks automation restart-media by anna [("note", "120")]"#,
            r#"disks workflow disks by anna [("target", Number(120))]"#,
        ]
    );
    let (unknown, _) = send(&api, "POST", "/api/widgets/disks/actions/forged", "").await;
    assert_eq!(unknown, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_button_needs_the_right_of_what_it_runs() {
    let family = Principal::member(
        "anna",
        Some("family".into()),
        Rights::of([Right::new(Area::Layout, Action::Update)]),
    );
    let api = widgets_api(family);
    let (status, _) = send(&api, "POST", "/api/widgets/disks/actions/restart", "").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let body = json!({ "settings": settings_of(&api, "clients"), "run": true }).to_string();
    let (preview, _) = send(&api, "POST", "/api/widgets/preview", &body).await;
    assert_eq!(preview, StatusCode::FORBIDDEN);
    assert!(api.runs.started.lock().unwrap().is_empty());
}

#[tokio::test]
async fn refreshing_runs_the_source_again_on_the_next_read() {
    let api = widgets_api(Principal::admin("admin"));
    data_of(&api, "backup").await.unwrap();
    assert!(!api.feature.custom.expired("backup"));
    let (status, body) = send(&api, "POST", "/api/widgets/backup/actions/refresh", "").await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["run_id"], Value::Null);
    assert!(api.feature.custom.expired("backup"));
    data_of(&api, "backup").await.unwrap();
    assert!(!api.feature.custom.expired("backup"));
    assert_eq!(api.runs.sources.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn a_preview_runs_a_new_source_answers_its_data_and_blocks_and_the_paths_it_declares() {
    let api = widgets_api(Principal::admin("admin"));
    let settings = json!({ "source": { "workflow": "disks" }, "blocks": [{ "kind": "progress", "value": "{{data.free}}", "maximum": "{{data.total}}", "thresholds": { "warning": 75, "danger": 90 } }] });
    let (status, body) = send(
        &api,
        "POST",
        "/api/widgets/preview",
        &json!({ "settings": settings, "run": true }).to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["ran"], true);
    assert_eq!(body["data"]["free"], 120);
    assert_eq!(body["blocks"][0]["value"], 120.0);
    assert_eq!(body["blocks"][0]["tone"], "danger");
    assert_eq!(body["blocks"][0]["maximum"], 500.0);
    let weather = json!({ "source": { "workflow": "weather" }, "blocks": [{ "kind": "text", "text": "{{data.temperature}}" }] });
    let (_, known) = send(
        &api,
        "POST",
        "/api/widgets/preview",
        &json!({ "settings": weather }).to_string(),
    )
    .await;
    assert_eq!(known["ran"], false);
    assert_eq!(
        known["paths"],
        json!([{ "path": "data.temperature", "description": "°C now", "kind": "number" }])
    );
    let sample = json!({ "settings": { "blocks": [{ "kind": "text", "text": "{{data.name}}" }] }, "sample": { "name": "nas" } });
    let (_, sampled) = send(&api, "POST", "/api/widgets/preview", &sample.to_string()).await;
    assert_eq!(sampled["blocks"][0]["text"], "nas");
}
