use std::collections::BTreeMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use axum::{Extension, Router};
use http_body_util::BodyExt;
use portal_feature::{Feature, Principal, WidgetProvider};
use portal_model::Environment;
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;

use crate::WidgetsFeature;
use crate::fakes::{FakeReferences, FakeRuns, FakeTemplates};
use crate::ports::DeclaredPath;
use crate::types::WidgetPorts;

pub const WIDGETS: &str = r#"[[dashboard.widgets]]
type = "custom"
id = "disks"
title = "Disks"
settings = { source = { workflow = "disks" }, blocks = [
  { kind = "stat", label = "Free", value = "{{data.free}} of {{data.total}} GB" },
  { kind = "button", label = "Restart", confirm = "Restart Jellyfin?", action = { id = "restart", automation = "restart-media", fields = { note = "{{data.free}}" } } },
  { kind = "button", label = "Refresh", action = { refresh = true } },
  { kind = "button", label = "Again", action = { id = "again", workflow = "disks", inputs = { target = "{{data.free}}" } } },
] }

[[dashboard.widgets]]
type = "custom"
id = "clients"
settings = { source = { script = "clients.sh" }, blocks = [{ kind = "list", items = "{{data.clients}}", text = "{{item.name}}", secondary = "{{item.ip}}" }] }

[[dashboard.widgets]]
type = "custom"
id = "backup"
public = true
settings = { source = { script = "backup.sh" }, blocks = [{ kind = "text", text = "{{data}}" }, { kind = "button", label = "Again", action = { refresh = true } }] }

[[dashboard.widgets]]
type = "custom"
id = "failing"
settings = { source = { workflow = "broken" }, blocks = [{ kind = "text", text = "never" }] }

[[dashboard.widgets]]
type = "custom"
id = "wifi"
settings = { blocks = [{ kind = "text", text = "Wi-Fi: home-5G" }] }
"#;

pub struct Api {
    pub router: Router,
    pub feature: WidgetsFeature,
    pub runs: Arc<FakeRuns>,
    pub _folder: TempDir,
}

fn references() -> FakeReferences {
    FakeReferences {
        workflows: BTreeMap::from([
            ("disks".to_string(), vec!["target".to_string()]),
            ("broken".to_string(), Vec::new()),
            ("weather".to_string(), Vec::new()),
        ]),
        scripts: vec!["clients.sh".into(), "backup.sh".into()],
        automations: BTreeMap::from([("restart-media".to_string(), vec!["note".to_string()])]),
        paths: BTreeMap::from([(
            "weather".to_string(),
            vec![DeclaredPath {
                path: "data.temperature".into(),
                description: Some("°C now".into()),
                kind: "number".into(),
            }],
        )]),
    }
}

fn runs() -> FakeRuns {
    FakeRuns {
        answers: BTreeMap::from([
            (
                "disks".to_string(),
                Ok(json!({ "free": 120, "total": 500, "token": "abc" })),
            ),
            (
                "clients.sh".to_string(),
                Ok(
                    json!({ "clients": [{ "name": "phone", "ip": "10.0.0.2" }, { "name": "tv", "ip": "10.0.0.3" }] }),
                ),
            ),
            (
                "backup.sh".to_string(),
                Ok(Value::String("backup ok, 3 h ago".into())),
            ),
            ("broken".to_string(), Err("the workflow failed".into())),
        ]),
        ..FakeRuns::default()
    }
}

pub fn widgets_api(principal: Principal) -> Api {
    let (folder, path) = portal_testing::written(WIDGETS);
    let configuration = Arc::new(portal_testing::opened(&path).unwrap());
    let runs = Arc::new(runs());
    let feature = WidgetsFeature::new(
        configuration,
        WidgetPorts {
            templates: Arc::new(FakeTemplates),
            references: Arc::new(references()),
            runs: runs.clone(),
        },
    );
    let router = feature
        .router()
        .layer(Extension(principal))
        .layer(Extension(Environment::parse("local").unwrap()));
    Api {
        router,
        feature,
        runs,
        _folder: folder,
    }
}

pub fn settings_of(api: &Api, id: &str) -> Value {
    api.feature.custom.instance(id).unwrap().settings
}

pub async fn data_of(api: &Api, id: &str) -> Result<Value, String> {
    api.feature
        .custom
        .data_of(id, &settings_of(api, id))
        .await
        .map_err(|problem| problem.to_string())
}

pub async fn send(api: &Api, method: &str, uri: &str, body: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = api.router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
