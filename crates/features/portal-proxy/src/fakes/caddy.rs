use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde_json::{Value, json};
use tokio::net::{TcpListener, UnixListener};

#[derive(Debug, Default)]
pub struct Recorded {
    pub configuration: Value,
    pub loads: usize,
    pub refuse: Option<String>,
    pub delay: Option<Duration>,
    pub content_types: Vec<String>,
}

#[derive(Clone, Default)]
pub struct Caddy {
    pub recorded: Arc<Mutex<Recorded>>,
}

impl Caddy {
    pub const ROOT: &'static str =
        "-----BEGIN CERTIFICATE-----\nMIIBfake\n-----END CERTIFICATE-----\n";

    pub async fn on_tcp() -> (Caddy, String) {
        let caddy = Caddy::default();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let router = caddy.router();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        (caddy, format!("http://{address}"))
    }

    pub async fn on_socket(path: &Path) -> Caddy {
        let caddy = Caddy::default();
        let listener = UnixListener::bind(path).unwrap();
        let router = caddy.router();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        caddy
    }

    pub fn with<T>(&self, change: impl FnOnce(&mut Recorded) -> T) -> T {
        change(&mut self.recorded.lock().unwrap_or_else(PoisonError::into_inner))
    }

    fn router(&self) -> Router {
        Router::new()
            .route("/load", post(load))
            .route("/config/", get(config))
            .route("/pki/ca/local", get(authority))
            .with_state(self.clone())
    }

    async fn pause(&self) {
        if let Some(delay) = self.with(|recorded| recorded.delay) {
            tokio::time::sleep(delay).await;
        }
    }
}

async fn load(State(caddy): State<Caddy>, headers: HeaderMap, Json(body): Json<Value>) -> Response {
    caddy.pause().await;
    let content_type = headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    caddy.with(|recorded| {
        recorded.content_types.push(content_type);
        if let Some(message) = &recorded.refuse {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": message }))).into_response();
        }
        recorded.configuration = body;
        recorded.loads += 1;
        StatusCode::OK.into_response()
    })
}

async fn config(State(caddy): State<Caddy>) -> Response {
    caddy.pause().await;
    Json(caddy.with(|recorded| recorded.configuration.clone())).into_response()
}

async fn authority(State(caddy): State<Caddy>) -> Response {
    caddy.pause().await;
    Json(json!({ "id": "local", "root_certificate": Caddy::ROOT })).into_response()
}
