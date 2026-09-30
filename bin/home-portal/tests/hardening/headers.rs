use axum::body::Body;
use axum::http::header::{
    CONTENT_SECURITY_POLICY, COOKIE, HOST, ORIGIN, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS,
};
use axum::http::{Request, StatusCode};

use super::portal::{PROBE_PATH, TICKET, WEBHOOK_PATH, answer};

#[tokio::test]
async fn every_answer_forbids_framing_and_sniffing() {
    for path in ["/", PROBE_PATH, "/health"] {
        let response = answer(
            Request::get(path)
                .header(COOKIE, TICKET)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        let headers = response.headers();
        assert_eq!(headers[X_CONTENT_TYPE_OPTIONS], "nosniff", "{path}");
        assert_eq!(headers[REFERRER_POLICY], "same-origin", "{path}");
        assert!(
            headers[CONTENT_SECURITY_POLICY]
                .to_str()
                .unwrap()
                .contains("frame-ancestors 'none'"),
            "{path}"
        );
    }
}

fn posted(path: &str, extra: &[(&str, &str)]) -> Request<Body> {
    let mut builder = Request::post(path)
        .header(COOKIE, TICKET)
        .header(HOST, "portal.home.example");
    for (name, value) in extra {
        builder = builder.header(*name, *value);
    }
    builder.body(Body::empty()).unwrap()
}

#[tokio::test]
async fn a_sibling_site_cannot_post_to_the_portal() {
    let from_wiki = posted(
        PROBE_PATH,
        &[
            (ORIGIN.as_str(), "https://wiki.home.example"),
            ("sec-fetch-site", "same-site"),
        ],
    );
    assert_eq!(answer(from_wiki).await.status(), StatusCode::FORBIDDEN);
    let origin_only = posted(
        PROBE_PATH,
        &[(ORIGIN.as_str(), "https://wiki.home.example")],
    );
    assert_eq!(answer(origin_only).await.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn the_portal_itself_and_command_line_clients_may_post() {
    let own = posted(
        PROBE_PATH,
        &[
            (ORIGIN.as_str(), "https://portal.home.example"),
            ("sec-fetch-site", "same-origin"),
        ],
    );
    assert_eq!(answer(own).await.status(), StatusCode::OK);
    assert_eq!(
        answer(posted(PROBE_PATH, &[])).await.status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn the_webhook_receiver_is_not_limited_to_the_portal_origin() {
    let hook = posted(
        WEBHOOK_PATH,
        &[
            (ORIGIN.as_str(), "https://ci.example"),
            ("sec-fetch-site", "cross-site"),
        ],
    );
    assert_eq!(answer(hook).await.status(), StatusCode::OK);
}
