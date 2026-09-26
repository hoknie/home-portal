use std::net::IpAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::DefaultBodyLimit;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hickory_proto::op::{Message, Query, ResponseCode};
use hickory_proto::rr::{Name, RecordType};
use http_body_util::BodyExt;
use portal_feature::ClientAddress;
use tower::ServiceExt;

use super::dns_query::DNS_MESSAGE;
use super::{resolve_encoded, resolve_posted};
use crate::services::Library;
use crate::services::tests::support::{book, published, settings};
use crate::types::Cadence;

fn router(from: &str) -> Router {
    let library = Arc::new(Library::default());
    library.publish(
        settings("[dns]\nenabled = true\nzones = [\"home\"]\n[dns.https]\nenabled = true\n"),
        book(
            "[dns]\nzones = [\"home\"]\n[dns.addresses]\nlocal = \"192.168.1.60\"\n",
            &[published("jellyfin.home", None)],
            &[],
        ),
    );
    let address: IpAddr = from.parse().unwrap();
    Router::new()
        .route("/dns-query", get(resolve_encoded).post(resolve_posted))
        .layer(DefaultBodyLimit::max(Cadence::LARGEST_MESSAGE))
        .layer(axum::Extension(ClientAddress(address)))
        .with_state(library)
}

fn question() -> Vec<u8> {
    let mut message = Message::new();
    message.set_id(0).add_query(Query::query(
        Name::from_ascii("jellyfin.home.").unwrap(),
        RecordType::A,
    ));
    message.to_vec().unwrap()
}

async fn send(router: Router, request: Request<Body>) -> (StatusCode, Option<String>, Vec<u8>) {
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let kind = response
        .headers()
        .get("content-type")
        .map(|value| value.to_str().unwrap().to_string());
    (
        status,
        kind,
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
}

#[tokio::test]
async fn a_phone_at_home_gets_the_home_address_over_https() {
    let encoded = URL_SAFE_NO_PAD.encode(question());
    let request = Request::get(format!("/dns-query?dns={encoded}"))
        .body(Body::empty())
        .unwrap();
    let (status, kind, body) = send(router("192.168.1.40"), request).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(kind.as_deref(), Some(DNS_MESSAGE));
    assert_eq!(
        Message::from_vec(&body).unwrap().answers()[0]
            .data()
            .to_string(),
        "192.168.1.60"
    );
    let posted = Request::post("/dns-query")
        .header("content-type", DNS_MESSAGE)
        .body(Body::from(question()))
        .unwrap();
    let (status, _, body) = send(router("192.168.1.40"), posted).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(Message::from_vec(&body).unwrap().answers().len(), 1);
}

#[tokio::test]
async fn dns_over_https_from_outside_is_a_refused_answer() {
    let posted = Request::post("/dns-query")
        .header("content-type", DNS_MESSAGE)
        .body(Body::from(question()))
        .unwrap();
    let (status, _, body) = send(router("203.0.113.7"), posted).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        Message::from_vec(&body).unwrap().response_code(),
        ResponseCode::Refused
    );
}

#[tokio::test]
async fn an_oversized_a_foreign_and_a_badly_encoded_request_are_refused() {
    let big = Request::post("/dns-query")
        .header("content-type", DNS_MESSAGE)
        .body(Body::from(vec![0u8; 5000]))
        .unwrap();
    assert_eq!(
        send(router("192.168.1.40"), big).await.0,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    let garbage = Request::post("/dns-query")
        .header("content-type", DNS_MESSAGE)
        .body(Body::from("hello"))
        .unwrap();
    assert_eq!(
        send(router("192.168.1.40"), garbage).await.0,
        StatusCode::BAD_REQUEST
    );
    let untyped = Request::post("/dns-query")
        .header("content-type", "text/plain")
        .body(Body::from(question()))
        .unwrap();
    assert_eq!(
        send(router("192.168.1.40"), untyped).await.0,
        StatusCode::BAD_REQUEST
    );
    let bad = Request::get("/dns-query?dns=***")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        send(router("192.168.1.40"), bad).await.0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn dns_over_https_is_not_served_while_it_is_off() {
    let library = Arc::new(Library::default());
    library.publish(settings("[dns]\nenabled = true\n"), book("", &[], &[]));
    let router = Router::new()
        .route("/dns-query", get(resolve_encoded).post(resolve_posted))
        .layer(axum::Extension(ClientAddress(
            "192.168.1.40".parse().unwrap(),
        )))
        .with_state(library);
    let encoded = URL_SAFE_NO_PAD.encode(question());
    let request = Request::get(format!("/dns-query?dns={encoded}"))
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(router, request).await.0, StatusCode::NOT_FOUND);
}
