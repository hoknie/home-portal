use std::fs;
use std::net::{IpAddr, TcpListener, UdpSocket};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use ipnet::IpNet;
use portal_config::ConfigStore;
use portal_feature::Feature;
use portal_model::{Environment, Environments};
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;

use super::DnsFeature;
use crate::ports::DnsSources;
use crate::types::{Cadence, PublishedHost};

struct Sources;

impl DnsSources for Sources {
    fn environments(&self) -> Environments {
        Environments::new(vec![(
            Environment::parse("local").unwrap(),
            vec!["127.0.0.0/8".parse::<IpNet>().unwrap()],
        )])
    }

    fn published(&self) -> Vec<PublishedHost> {
        Vec::new()
    }

    fn interfaces(&self) -> Vec<IpAddr> {
        Vec::new()
    }
}

const QUICK: Cadence = Cadence {
    tick: Duration::from_millis(50),
    retry: Duration::from_millis(200),
    interfaces: Duration::from_secs(60),
    idle: Duration::from_millis(300),
};

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn portal(text: &str) -> (TempDir, Arc<ConfigStore>, Router) {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    let configuration = Arc::new(ConfigStore::open(&path).unwrap());
    let feature = DnsFeature::with(configuration.clone(), Arc::new(Sources), QUICK);
    configuration
        .adopt(vec![feature.validator().unwrap()])
        .unwrap();
    for task in feature.loops() {
        tokio::spawn(task);
    }
    (folder, configuration, feature.router())
}

async fn get(router: &Router) -> (Value, String) {
    let response = router
        .clone()
        .oneshot(Request::get(DnsFeature::PATH).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let etag = response.headers()["etag"].to_str().unwrap().to_string();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (serde_json::from_slice(&bytes).unwrap(), etag)
}

async fn settled(router: &Router, check: impl Fn(&Value) -> bool) -> Value {
    let began = Instant::now();
    loop {
        let (body, _) = get(router).await;
        if check(&body) || began.elapsed() > Duration::from_secs(5) {
            return body;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn enabled_on(port: u16) -> String {
    format!(
        "[environments.local]\nnetworks = [\"127.0.0.0/8\"]\n\n# Local names.\n[dns]\nenabled = true # on\naddress = \"127.0.0.1\"\nport = {port}\nzones = [\"home\"]\n"
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn off_by_default_nothing_listens() {
    let (_folder, _configuration, router) = portal("");
    let body = settled(&router, |body| body["plain"]["reason"].is_string()).await;
    assert_eq!(body["enabled"], false);
    assert_eq!(body["plain"]["listening"], false);
    assert_eq!(body["https"]["listening"], false);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_taken_port_is_reported_and_the_rest_keeps_running() {
    let holder = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = holder.local_addr().unwrap().port();
    let (_folder, _configuration, router) = portal(&enabled_on(port));
    let body = settled(&router, |body| {
        body["plain"]["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains(&port.to_string()))
    })
    .await;
    assert_eq!(body["plain"]["listening"], false);
    assert!(body["last_error"].is_string());
    assert_eq!(body["zones"][0]["apex"], "home");
}

#[tokio::test(flavor = "multi_thread")]
async fn changing_the_port_through_the_api_moves_the_server_and_keeps_comments() {
    let (first, second) = (free_port(), free_port());
    let (folder, _configuration, router) = portal(&enabled_on(first));
    let (_, etag) = get(&router).await;
    let body = format!(
        "{{\"enabled\":true,\"address\":\"127.0.0.1\",\"port\":{second},\"zones\":[\"home\"],\"ttl\":60,\"addresses\":{{}},\"tls\":{{\"enabled\":false}},\"https\":{{\"enabled\":false}}}}"
    );
    let request = Request::put(DnsFeature::PATH)
        .header("content-type", "application/json")
        .header("if-match", etag)
        .body(Body::from(body))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let moved = settled(&router, |body| {
        body["plain"]["address"] == format!("127.0.0.1:{second}")
            && body["plain"]["listening"] == true
    })
    .await;
    assert_eq!(moved["settings"]["port"], second);
    let text = fs::read_to_string(folder.path().join("home-portal.toml")).unwrap();
    assert!(
        text.contains("# Local names.\n[dns]\nenabled = true # on\n"),
        "{text}"
    );
    assert!(UdpSocket::bind(("127.0.0.1", first)).is_ok());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_bad_port_through_the_api_names_the_field() {
    let (_folder, _configuration, router) = portal(&enabled_on(free_port()));
    let (_, etag) = get(&router).await;
    let request = Request::put(DnsFeature::PATH)
        .header("content-type", "application/json")
        .header("if-match", etag)
        .body(Body::from(
            "{\"enabled\":true,\"address\":\"127.0.0.1\",\"port\":853,\"zones\":[],\"ttl\":60}",
        ))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&bytes).contains("dns.tls.port"));
}
