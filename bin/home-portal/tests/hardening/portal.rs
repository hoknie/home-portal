use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::header::COOKIE;
use axum::http::{HeaderMap, Method, Request, Response};
use axum::routing::{get, post};
use home_portal::{Registry, Restart, assemble};
use portal_feature::{ApiError, Feature, Gate, Principal, Rule};
use tower::ServiceExt;

pub const PROBE_PATH: &str = "/api/probe";
pub const WEBHOOK_PATH: &str = "/webhook/probe";
pub const TICKET: &str = "ticket=yes";
pub const EXAMPLE: &str = "config/home-portal.example.toml";

struct ProbeFeature;

impl Feature for ProbeFeature {
    fn name(&self) -> &'static str {
        "probe"
    }

    fn router(&self) -> Router {
        Router::new().route(
            PROBE_PATH,
            get(|| async { "probe" }).post(|| async { "posted" }),
        )
    }

    fn rules(&self) -> Vec<Rule> {
        vec![
            Rule::signed(Method::GET, PROBE_PATH),
            Rule::signed(Method::POST, PROBE_PATH),
        ]
    }

    fn public_router(&self) -> Router {
        Router::new().route(WEBHOOK_PATH, post(|| async { "received" }))
    }
}

struct Ticket;

impl Gate for Ticket {
    fn admit(&self, headers: &HeaderMap) -> Result<Principal, ApiError> {
        match headers.get(COOKIE).and_then(|value| value.to_str().ok()) {
            Some(TICKET) => Ok(Principal::admin("tester")),
            _ => Err(ApiError::Unauthorized),
        }
    }
}

struct Silent;

#[async_trait::async_trait]
impl portal_feature::EventSink for Silent {
    fn emit(&self, _event: portal_feature::PortalEvent) {}

    async fn settle(&self, _within: std::time::Duration) {}
}

pub fn probe_portal() -> Router {
    let directory = Box::leak(Box::new(tempfile::tempdir().unwrap()));
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let example = std::fs::read_to_string(root.join(EXAMPLE)).unwrap();
    let hash = portal_auth::hash_password("secret").unwrap();
    let path = directory.path().join("home-portal.toml");
    std::fs::write(
        &path,
        format!("{example}\n[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\ngroup = \"admin\"\n"),
    )
    .unwrap();
    let configuration = Arc::new(portal_config::ConfigStore::open(path).unwrap());
    let interface: Arc<dyn portal_web::AssetSource> =
        Arc::new(portal_web::Directory::first_of(&[std::path::Path::new(
            env!("CARGO_MANIFEST_DIR"),
        )
        .join("tests/fixtures/web")]));
    assemble(&Registry {
        features: vec![
            Arc::new(ProbeFeature),
            Arc::new(portal_health::HealthFeature),
        ],
        gate: Arc::new(Ticket),
        configuration: configuration.clone(),
        widgets: Arc::new(portal_widget::WidgetRegistry::new(
            configuration,
            Vec::new(),
        )),
        events: Arc::new(Silent),
        restart: Restart::default(),
        interface,
    })
}

pub async fn answer(request: Request<Body>) -> Response<Body> {
    probe_portal().oneshot(request).await.unwrap()
}
