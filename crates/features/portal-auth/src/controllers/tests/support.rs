use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::http::header::{CONTENT_TYPE, COOKIE, ETAG, IF_MATCH, SET_COOKIE};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use http_body_util::BodyExt;
use portal_config::ConfigStore;
use portal_feature::{EventSink, Feature, Gate, PortalEvent};
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;

use crate::features::AuthFeature;
use crate::helpers::hash_password;
use crate::ports::Connection;
use crate::types::CookieScope;

pub const ON: &str = "[modules]\nusers = true\n\n";
pub const OFF: &str = "";

struct Direct;

impl Connection for Direct {
    fn client_address(&self, _peer: Option<SocketAddr>, _headers: &HeaderMap) -> IpAddr {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    }

    fn cookie_scope(&self, _peer: Option<SocketAddr>, _headers: &HeaderMap) -> CookieScope {
        CookieScope::default()
    }
}

struct Silent;

#[async_trait]
impl EventSink for Silent {
    fn emit(&self, _event: PortalEvent) {}

    async fn settle(&self, _within: Duration) {}
}

pub struct Portal {
    pub router: Router,
    pub folder: TempDir,
    pub main: PathBuf,
    pub gate: Arc<dyn Gate>,
}

async fn require(gate: Arc<dyn Gate>, mut request: Request<Body>, next: Next) -> Response {
    match gate.admit(request.headers()) {
        Ok(principal) => {
            request.extensions_mut().insert(principal);
            next.run(request).await
        }
        Err(error) => error.into_response(),
    }
}

pub fn entry(name: &str, password: &str) -> String {
    member(name, password, Some("admin"))
}

pub fn member(name: &str, password: &str, group: Option<&str>) -> String {
    let group = group.map_or(String::new(), |group| format!("group = \"{group}\"\n"));
    format!(
        "[[users]]\nname = \"{name}\"\npassword_hash = \"{}\"\n{group}",
        hash_password(password).unwrap()
    )
}

pub fn portal_with(files: &[(&str, String)]) -> Portal {
    let folder = tempfile::tempdir().unwrap();
    for (name, text) in files {
        fs::write(folder.path().join(name), text).unwrap();
    }
    let main = folder.path().join(files[0].0);
    let store = Arc::new(ConfigStore::open(&main).unwrap());
    let feature = AuthFeature::new(store.clone(), Arc::new(Direct), Arc::new(Silent));
    store
        .adopt(vec![
            feature.validator().unwrap(),
            portal_feature::ModuleSwitches::errors,
        ])
        .unwrap();
    let gate = feature.gate();
    let checking = gate.clone();
    let router = feature
        .router()
        .route_layer(middleware::from_fn(move |request, next| {
            require(checking.clone(), request, next)
        }))
        .merge(feature.public_router());
    Portal {
        router,
        folder,
        main,
        gate,
    }
}

pub fn portal(text: String) -> Portal {
    portal_with(&[("home-portal.toml", text)])
}

pub async fn send(portal: &Portal, request: Request<Body>) -> (StatusCode, Option<String>, Value) {
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

pub async fn signed_in(portal: &Portal, name: &str, password: &str) -> Option<String> {
    let response = portal
        .router
        .clone()
        .oneshot(
            Request::post(AuthFeature::PATH)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(format!(
                    r#"{{"name":"{name}","password":"{password}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    (response.status() == StatusCode::OK).then(|| {
        response.headers()[SET_COOKIE]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_string()
    })
}

pub fn cookie_headers(cookie: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookie.parse().unwrap());
    headers
}

pub fn get(uri: &str, cookie: &str) -> Request<Body> {
    Request::get(uri)
        .header(COOKIE, cookie)
        .body(Body::empty())
        .unwrap()
}

pub fn write(method: &str, uri: &str, cookie: &str, revision: &str, body: &str) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(COOKIE, cookie)
        .header(IF_MATCH, revision);
    if !body.is_empty() {
        builder = builder.header(CONTENT_TYPE, "application/json");
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

pub async fn revision(portal: &Portal, cookie: &str) -> String {
    send(portal, get(AuthFeature::USERS, cookie))
        .await
        .1
        .unwrap()
}

pub fn text_of(portal: &Portal) -> String {
    fs::read_to_string(&portal.main).unwrap()
}

pub fn users_text_of(portal: &Portal) -> String {
    fs::read_to_string(portal.folder.path().join("users.toml")).unwrap_or_default()
}
