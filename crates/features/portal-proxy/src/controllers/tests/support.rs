use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::response::Response;
use http_body_util::BodyExt;
use portal_config::ConfigStore;
use portal_feature::Feature;
use portal_model::{DetectedEnvironment, Environment, Environments};
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;

use crate::ProxyFeature;
use crate::fakes::{Catalogue, Loopback, Sessions};
use crate::services::CaddyManager;
use crate::types::{CaddyHome, Cadence, ProxyPorts, SyncSetup};

pub const CADDY: &str = "127.0.0.1";
pub const OUTSIDE: &str = "203.0.113.9";

pub struct Portal {
    pub router: Router,
    pub path: std::path::PathBuf,
    _directory: TempDir,
}

pub fn file(admin: &str) -> String {
    format!(
        "[network]\ntrusted_proxies = [\"127.0.0.1\"]\n\n[proxy]\nenabled = true\nadmin = \"{admin}\"\nportal_host = \"portal.example.com\"\ncookie_domain = \"example.com\"\n\n[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://192.168.1.5\"\nproxy = {{ host = \"nas.example.com\", auth = [\"internet\"] }}\n\n[[services]]\nid = \"media\"\nname = \"Media\"\nurl = \"http://192.168.1.10:8096\"\nproxy = {{ host = \"media.home.arpa\", tls = {{ mode = \"internal\" }} }}\n"
    )
}

pub fn portal(text: &str) -> Portal {
    portal_with(text, "http://127.0.0.1:9")
}

pub fn portal_with(text: &str, source: &str) -> Portal {
    portal_pinned(text, source, "latest")
}

pub fn portal_pinned(text: &str, source: &str, version: &str) -> Portal {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    let text = text.replacen(
        "[proxy]\n",
        &format!("[proxy]\ncaddy = {{ source = \"{source}\", version = \"{version}\" }}\n"),
        1,
    );
    fs::write(&path, text).unwrap();
    let configuration = Arc::new(ConfigStore::open(&path).unwrap());
    configuration.adopt(vec![ProxyFeature::validate]).unwrap();
    let setup = SyncSetup {
        portal: "127.0.0.1:8080".parse().unwrap(),
        cadence: Cadence::STANDARD,
        manager: Arc::new(CaddyManager::new(CaddyHome::at(
            path.with_file_name("caddy"),
        ))),
    };
    let feature = ProxyFeature::with(
        configuration.clone(),
        ProxyPorts {
            services: Arc::new(Catalogue { configuration }),
            gate: Arc::new(Sessions),
            peers: Arc::new(Loopback),
        },
        setup,
    );
    Portal {
        router: feature.router().merge(feature.public_router()),
        path,
        _directory: directory,
    }
}

pub async fn send(
    portal: &Portal,
    request: Request<Body>,
    peer: &str,
    environment: &str,
) -> Response {
    let mut request = request;
    let peer: SocketAddr = format!("{peer}:40000").parse().unwrap();
    request.extensions_mut().insert(ConnectInfo(peer));
    let environment = Environment::parse(environment).unwrap();
    let environments = Environments::new(vec![(Environment::parse("local").unwrap(), Vec::new())]);
    request
        .extensions_mut()
        .insert(DetectedEnvironment::new(environment, &environments));
    portal.router.clone().oneshot(request).await.unwrap()
}

pub async fn json(response: Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

pub fn authorize(host: &str, cookie: Option<&str>, method: &str) -> Request<Body> {
    let mut builder = Request::get("/api/proxy/authorize")
        .header("x-forwarded-host", host)
        .header("x-forwarded-uri", "/photos")
        .header("x-forwarded-method", method);
    if let Some(cookie) = cookie {
        builder = builder.header("cookie", cookie);
    }
    builder.body(Body::empty()).unwrap()
}

pub fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

pub fn location(response: &Response) -> &str {
    response.headers()["location"].to_str().unwrap()
}
