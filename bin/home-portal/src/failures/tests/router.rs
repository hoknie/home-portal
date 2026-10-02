use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Extension;
use axum::body::{Body, to_bytes};
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, Request, StatusCode};
use portal_web::Directory;
use tower::ServiceExt;

use super::support::{HOST, LOCAL, OUTSIDE, invalid, main_of};
use crate::failures::board::FailureBoard;
use crate::failures::main_file::MainFile;
use crate::failures::problems::problems_of;
use crate::failures::router::failure_router;

fn interface() -> (tempfile::TempDir, Arc<Directory>) {
    let folder = tempfile::tempdir().unwrap();
    for page in ["en/index.html", "en/fatal/index.html"] {
        let path = folder.path().join(page);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, format!("<html>{page}</html>")).unwrap();
    }
    let directory = Arc::new(Directory::first_of(&[folder.path().to_path_buf()]));
    (folder, directory)
}

async fn ask(
    main: MainFile,
    from: &str,
    method: &str,
    path: &str,
) -> (StatusCode, HeaderMap, String) {
    let (_folder, directory) = interface();
    let board = FailureBoard::new(problems_of(&invalid(&["users"])), main.clone());
    let router = failure_router(board, directory, main.language);
    let request = Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .unwrap();
    let address: SocketAddr = from.parse().unwrap();
    let response = router
        .layer(Extension(ConnectInfo(address)))
        .oneshot(request)
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, headers, String::from_utf8_lossy(&body).to_string())
}

#[tokio::test]
async fn health_answers_503_failed() {
    let (status, headers, body) = ask(MainFile::defaults(), HOST, "GET", "/health").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, "failed");
    assert!(
        headers["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/plain")
    );
}

#[tokio::test]
async fn the_report_lists_the_problems_for_the_host() {
    let (status, _, body) = ask(MainFile::defaults(), HOST, "GET", "/api/portal/failure").await;
    assert_eq!(status, StatusCode::OK);
    let report: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(report["details"], true);
    assert_eq!(report["problems"][0]["field"], "users");
    assert!(report["since"].is_string() && report["checked"].is_string());
}

#[tokio::test]
async fn the_report_hides_the_problems_from_outside() {
    let (_, _, body) = ask(main_of(LOCAL), OUTSIDE, "GET", "/api/portal/failure").await;
    let report: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(report["details"], false);
    assert!(report["problems"].is_null());
    assert!(!body.contains("home-portal.toml"));
}

#[tokio::test]
async fn any_other_api_call_answers_503_with_the_state_header() {
    for path in ["/api/services", "/api", "/api/portal/restart"] {
        let (status, headers, body) = ask(MainFile::defaults(), HOST, "GET", path).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{path}");
        assert_eq!(headers["portal-state"], "failed", "{path}");
        assert!(body.contains("/fatal/"), "{path}");
    }
}

#[tokio::test]
async fn a_page_redirects_to_the_failure_page() {
    for path in ["/", "/admin/services/", "/login/", "/fatal"] {
        let (status, headers, _) = ask(MainFile::defaults(), HOST, "GET", path).await;
        assert_eq!(status, StatusCode::TEMPORARY_REDIRECT, "{path}");
        assert_eq!(headers["location"], "/fatal/", "{path}");
    }
}

#[tokio::test]
async fn the_failure_page_and_files_are_served_with_the_security_headers() {
    let (status, headers, body) = ask(MainFile::defaults(), HOST, "GET", "/fatal/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("en/fatal/index.html"), "{body}");
    assert_eq!(headers["x-content-type-options"], "nosniff");
    assert!(
        headers["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("frame-ancestors 'none'")
    );
    let (status, _, _) = ask(
        MainFile::defaults(),
        HOST,
        "GET",
        "/_next/static/missing.js",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
