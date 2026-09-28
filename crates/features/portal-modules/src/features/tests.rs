use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::header::{CONTENT_TYPE, ETAG, IF_MATCH};
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use portal_config::ConfigStore;
use portal_feature::{Feature, FieldError, Module, ModulePreparer, ModuleSwitches};
use serde_json::Value;
use tempfile::TempDir;
use toml_edit::DocumentMut;
use tower::ServiceExt;

use super::ModulesFeature;

struct TrustLoopback;

impl ModulePreparer for TrustLoopback {
    fn module(&self) -> Module {
        Module::Proxy
    }

    fn touches(&self) -> &'static [&'static str] {
        &["network"]
    }

    fn prepare(&self, document: &mut DocumentMut) {
        document["network"]["trusted_proxies"] =
            toml_edit::value(toml_edit::Array::from_iter(["127.0.0.1"]));
    }
}

fn proxy_needs_a_host(document: &DocumentMut) -> Vec<FieldError> {
    let on = ModuleSwitches::resolve(document)
        .unwrap_or_default()
        .is_on(Module::Proxy);
    let host = document
        .get("proxy")
        .and_then(|proxy| proxy.get("portal_host"))
        .is_some();
    if on && !host {
        vec![FieldError::new("proxy.portal_host", "is required")]
    } else {
        Vec::new()
    }
}

struct Portal {
    router: Router,
    path: PathBuf,
    folder: TempDir,
}

fn portal_with(files: &[(&str, &str)]) -> Portal {
    let folder = TempDir::new().unwrap();
    for (name, text) in files {
        fs::write(folder.path().join(name), text).unwrap();
    }
    let path = folder.path().join(files[0].0);
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let feature = ModulesFeature::new(store.clone(), vec![Arc::new(TrustLoopback)]);
    store
        .adopt(vec![feature.validator().unwrap(), proxy_needs_a_host])
        .unwrap();
    Portal {
        router: feature.router(),
        path,
        folder,
    }
}

fn portal(text: &str) -> Portal {
    portal_with(&[("home-portal.toml", text)])
}

async fn answer(portal: &Portal, request: Request<Body>) -> (StatusCode, Option<String>, Value) {
    let response = portal.router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let etag = response
        .headers()
        .get(ETAG)
        .map(|value| value.to_str().unwrap().to_string());
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).to_string()));
    (status, etag, body)
}

async fn modules(portal: &Portal) -> (String, Value) {
    let (status, etag, body) = answer(
        portal,
        Request::get(ModulesFeature::PATH)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    (etag.unwrap(), body)
}

fn switch(name: &str, enabled: bool, revision: Option<&str>) -> Request<Body> {
    let mut builder =
        Request::put(format!("/api/modules/{name}")).header(CONTENT_TYPE, "application/json");
    if let Some(revision) = revision {
        builder = builder.header(IF_MATCH, revision);
    }
    builder
        .body(Body::from(format!("{{\"enabled\":{enabled}}}")))
        .unwrap()
}

async fn switched(portal: &Portal, name: &str, enabled: bool) -> (StatusCode, Value) {
    let (revision, _) = modules(portal).await;
    let (status, _, body) = answer(portal, switch(name, enabled, Some(&revision))).await;
    (status, body)
}

fn entry<'a>(body: &'a Value, name: &str) -> &'a Value {
    body["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|module| module["name"] == name)
        .unwrap()
}

fn text_of(portal: &Portal) -> String {
    fs::read_to_string(&portal.path).unwrap()
}

#[tokio::test]
async fn every_module_is_listed_in_order_with_its_requirements() {
    let portal = portal("");
    let (_, body) = modules(&portal).await;
    let names: Vec<&str> = body["modules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|module| module["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        vec![
            "proxy",
            "dns",
            "automations",
            "webhooks",
            "users",
            "workflows",
            "notifications"
        ]
    );
    assert_eq!(entry(&body, "notifications")["enabled"], true);
    assert_eq!(entry(&body, "users")["enabled"], false);
    assert_eq!(entry(&body, "users")["requires"], serde_json::json!([]));
    assert_eq!(entry(&body, "proxy")["enabled"], false);
    assert_eq!(
        entry(&body, "dns")["requires"],
        serde_json::json!(["proxy"])
    );
    assert_eq!(
        entry(&body, "automations")["required_by"],
        serde_json::json!(["webhooks"])
    );
}

#[tokio::test]
async fn required_by_lists_only_enabled_modules() {
    let portal = portal("[modules]\nwebhooks = false\n");
    let (_, body) = modules(&portal).await;
    assert_eq!(
        entry(&body, "automations")["required_by"],
        serde_json::json!([])
    );
}

#[tokio::test]
async fn an_unknown_module_is_not_found() {
    let portal = portal("");
    let (status, _) = switched(&portal, "telegram", true).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn switching_automations_off_while_webhooks_are_on_is_refused() {
    let portal = portal("# mine\n");
    let (status, body) = switched(&portal, "automations", false).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body.as_str().unwrap().contains("webhooks"), "{body}");
    assert_eq!(text_of(&portal), "# mine\n");
}

#[tokio::test]
async fn switching_dns_on_while_the_proxy_is_off_is_refused() {
    let portal = portal("");
    let (status, body) = switched(&portal, "dns", true).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body.as_str().unwrap().contains("proxy"), "{body}");
    assert_eq!(text_of(&portal), "");
}

#[tokio::test]
async fn switching_the_proxy_on_without_a_host_names_the_field() {
    let portal = portal("[network]\nport = 8080\n");
    let (status, body) = switched(&portal, "proxy", true).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["errors"][0]["field"], "proxy.portal_host");
    assert_eq!(text_of(&portal), "[network]\nport = 8080\n");
}

#[tokio::test]
async fn switching_the_proxy_on_writes_the_switch_and_prepares_the_network() {
    let portal = portal("[proxy]\nportal_host = \"portal.example.com\"\n");
    let (status, body) = switched(&portal, "proxy", true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(entry(&body, "proxy")["enabled"], true);
    let text = text_of(&portal);
    assert!(text.contains("[modules]\nproxy = true"), "{text}");
    assert!(text.contains("trusted_proxies = [\"127.0.0.1\"]"), "{text}");
}

#[tokio::test]
async fn a_network_in_another_file_is_not_prepared_there() {
    let portal = portal_with(&[
        (
            "home-portal.toml",
            "include = [\"network.toml\"]\n\n[proxy]\nportal_host = \"portal.example.com\"\n",
        ),
        ("network.toml", "[network]\nport = 8080\n"),
    ]);
    let (status, _) = switched(&portal, "proxy", true).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!text_of(&portal).contains("trusted_proxies"));
    assert_eq!(
        fs::read_to_string(portal.folder.path().join("network.toml")).unwrap(),
        "[network]\nport = 8080\n"
    );
}

#[tokio::test]
async fn a_legacy_key_is_replaced_and_comments_are_kept() {
    let portal = portal(
        "[modules]\nproxy = true\n\n[proxy]\nportal_host = \"portal.example.com\"\n\n# names\n[dns]\nenabled = true # on\nport = 53\n",
    );
    let (status, body) = switched(&portal, "dns", false).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(entry(&body, "dns")["enabled"], false);
    let text = text_of(&portal);
    assert!(
        text.contains("[modules]\nproxy = true\ndns = false\n"),
        "{text}"
    );
    assert!(text.contains("# names\n[dns]\nport = 53\n"), "{text}");
}

#[tokio::test]
async fn a_stale_or_missing_revision_is_refused() {
    let portal = portal("");
    let (missing, _, _) = answer(&portal, switch("proxy", false, None)).await;
    assert_eq!(missing, StatusCode::PRECONDITION_REQUIRED);
    let (stale, _, _) = answer(&portal, switch("proxy", false, Some("\"old\""))).await;
    assert_eq!(stale, StatusCode::CONFLICT);
}

#[test]
fn a_file_with_dns_on_and_the_proxy_off_is_refused_as_modules_dns() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("home-portal.toml");
    fs::write(&path, "[dns]\nenabled = true\n").unwrap();
    let store = ConfigStore::open(&path).unwrap();
    let feature = ModulesFeature::new(Arc::new(ConfigStore::open(&path).unwrap()), Vec::new());
    let error = store
        .adopt(vec![feature.validator().unwrap()])
        .unwrap_err()
        .to_string();
    assert!(error.contains("modules.dns"), "{error}");
}
