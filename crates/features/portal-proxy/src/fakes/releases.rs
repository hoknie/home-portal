use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use axum::Router;
use axum::extract::{Path as Segment, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde_json::json;
use sha2::{Digest, Sha512};
use tokio::net::TcpListener;

use crate::types::Platform;

pub const VERSION: &str = "9.9.9";
pub const SCRIPT: &str = "#!/bin/sh\necho \"$$ $(ps -o pgid= -p $$ | tr -d ' ') $XDG_DATA_HOME $*\" >> launches.txt\nexec sleep 30\n";

#[derive(Clone, Default)]
pub struct Published {
    pub archive: Vec<u8>,
    pub checksums: String,
    pub delay: Option<Duration>,
    pub base: Arc<Mutex<String>>,
}

pub struct Releases {
    pub base: String,
}

impl Releases {
    pub fn archive_of(script: &str) -> Vec<u8> {
        let directory = tempfile::tempdir().unwrap();
        let binary = directory.path().join("caddy");
        fs::write(&binary, script).unwrap();
        make_executable(&binary);
        let archive = directory.path().join("caddy.tar.gz");
        let status = Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(directory.path())
            .arg("caddy")
            .status()
            .unwrap();
        assert!(status.success());
        fs::read(archive).unwrap()
    }

    pub async fn serve(archive: Vec<u8>, tamper: bool, delay: Option<Duration>) -> Releases {
        let platform = Platform::current().unwrap();
        let digest: String = Sha512::digest(&archive)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let digest = if tamper {
            digest.replace('a', "b")
        } else {
            digest
        };
        let published = Published {
            checksums: format!("{digest}  {}\n", platform.archive(VERSION)),
            archive,
            delay,
            base: Arc::default(),
        };
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        *published
            .base
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = base.clone();
        let router = Router::new()
            .route("/latest", get(latest))
            .route("/tags/{tag}", get(tagged))
            .route("/archive", get(archive_of))
            .route("/checksums", get(checksums))
            .with_state(published);
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Releases { base }
    }
}

async fn latest(State(published): State<Published>) -> Response {
    if let Some(delay) = published.delay {
        tokio::time::sleep(delay).await;
    }
    let platform = Platform::current().unwrap();
    let base = published
        .base
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    axum::Json(json!({
        "tag_name": format!("v{VERSION}"),
        "assets": [
            { "name": platform.archive(VERSION), "browser_download_url": format!("{base}/archive") },
            { "name": Platform::checksums(VERSION), "browser_download_url": format!("{base}/checksums") }
        ]
    }))
    .into_response()
}

async fn tagged(Segment(tag): Segment<String>, state: State<Published>) -> Response {
    if tag == format!("v{VERSION}") {
        latest(state).await
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn archive_of(State(published): State<Published>) -> Response {
    published.archive.into_response()
}

async fn checksums(State(published): State<Published>) -> Response {
    published.checksums.into_response()
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}
