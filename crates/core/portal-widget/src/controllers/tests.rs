use std::fs;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use portal_config::ConfigStore;
use portal_feature::FieldError;
use portal_model::Environment;
use serde_json::{Value, json};
use tower::ServiceExt;

use super::widgets_router;
use crate::services::WidgetRegistry;
use portal_feature::WidgetProblem;
use portal_feature::WidgetProvider;

struct Clock;

#[async_trait]
impl WidgetProvider for Clock {
    fn kind(&self) -> &'static str {
        "clock"
    }

    fn refresh(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn check(&self, _settings: &Value) -> Vec<FieldError> {
        Vec::new()
    }

    async fn data(&self, _settings: &Value) -> Result<Value, WidgetProblem> {
        Ok(json!({ "hour": 9 }))
    }
}

async fn ask(id: &str, environment: Environment) -> (StatusCode, Value) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(
        &path,
        "[[dashboard.widgets]]\ntype = \"clock\"\nid = \"clock\"\n\n[[dashboard.widgets]]\ntype = \"clock\"\nid = \"private\"\nenvironments = [\"local\"]\n",
    )
    .unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let registry = Arc::new(WidgetRegistry::new(store, vec![Arc::new(Clock)]));
    let mut request = Request::get(format!("/api/widgets/{id}/data"))
        .body(Body::empty())
        .unwrap();
    request.extensions_mut().insert(environment);
    let response = widgets_router(registry).oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn a_widget_answers_with_its_data_and_when_it_was_gathered() {
    let (status, body) = ask("clock", Environment::internet()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["hour"], 9);
    assert_eq!(body["stale"], false);
    assert_eq!(body["refresh_seconds"], 30);
    assert!(body["fetched_at"].as_str().unwrap().ends_with('Z'));
}

#[tokio::test]
async fn an_unknown_id_and_a_widget_of_another_environment_answer_the_same() {
    let (unknown, _) = ask("nope", Environment::internet()).await;
    let (hidden, _) = ask("private", Environment::internet()).await;
    assert_eq!(unknown, StatusCode::NOT_FOUND);
    assert_eq!(hidden, StatusCode::NOT_FOUND);
    let (visible, _) = ask("private", Environment::parse("local").unwrap()).await;
    assert_eq!(visible, StatusCode::OK);
}
