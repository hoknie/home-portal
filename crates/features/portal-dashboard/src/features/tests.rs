use std::fs;
use std::sync::Arc;

use axum::body::Body;
use axum::http::header::ETAG;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use portal_config::ConfigStore;
use portal_feature::Feature;
use portal_model::Environment;
use serde_json::Value;
use tower::ServiceExt;

use super::DashboardFeature;

async fn ask(text: &str, uri: &str) -> (StatusCode, bool, Value) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    let feature = DashboardFeature::new(Arc::new(ConfigStore::open(&path).unwrap()));
    let mut request = Request::get(uri).body(Body::empty()).unwrap();
    request
        .extensions_mut()
        .insert(Environment::parse("local").unwrap());
    let response = feature.router().oneshot(request).await.unwrap();
    let status = response.status();
    let tagged = response.headers().contains_key(ETAG);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, tagged, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn the_layout_is_served_with_a_revision_sections_and_keyed_widgets() {
    let (status, tagged, body) = ask(
        "[[dashboard.widgets]]\ntype = \"weather\"\nid = \"riga\"\nsize = \"half\"\n\n[[dashboard.widgets]]\ntype = \"status-summary\"\n",
        DashboardFeature::PATH,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(tagged);
    assert_eq!(
        body["sections"],
        serde_json::json!([{ "id": "main", "title": null }])
    );
    assert_eq!(body["widgets"][0]["key"], "riga");
    assert_eq!(body["widgets"][0]["size"], "half");
    assert_eq!(body["widgets"][0]["section"], "main");
    assert_eq!(body["widgets"][1]["key"], "#1");
    assert_eq!(body["widgets"][1]["size"], "full");
}

#[tokio::test]
async fn a_widget_of_another_environment_is_not_served_here_but_the_editor_sees_it() {
    let text = "[[dashboard.widgets]]\ntype = \"status-summary\"\nenvironments = [\"vpn\"]\n\n[[dashboard.widgets]]\ntype = \"services\"\n";
    let (_, _, body) = ask(text, DashboardFeature::PATH).await;
    let widgets = body["widgets"].as_array().unwrap();
    assert_eq!(widgets.len(), 1);
    assert_eq!(widgets[0]["key"], "#1");
    let (_, _, all) = ask(text, "/api/dashboard?all=true").await;
    let widgets = all["widgets"].as_array().unwrap();
    assert_eq!(widgets.len(), 2);
    assert_eq!(widgets[0]["environments"], serde_json::json!(["vpn"]));
}
