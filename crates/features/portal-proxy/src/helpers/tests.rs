use axum::http::HeaderMap;

use super::{continuation, forwarded_host, sign_in_address};

fn known() -> Vec<String> {
    vec!["portal.example.com".into(), "nas.example.com".into()]
}

#[test]
fn the_sign_in_address_carries_the_whole_original_address_encoded() {
    assert_eq!(
        sign_in_address(
            "https://portal.example.com",
            "https://nas.example.com",
            Some("/photos")
        ),
        "https://portal.example.com/login/?return=https%3A%2F%2Fnas.example.com%2Fphotos"
    );
    assert_eq!(
        sign_in_address(
            "https://portal.example.com",
            "https://nas.example.com",
            None
        ),
        "https://portal.example.com/login/?return=https%3A%2F%2Fnas.example.com%2F"
    );
}

#[test]
fn a_forwarded_host_loses_its_port_and_case() {
    let mut headers = HeaderMap::new();
    headers.insert("x-forwarded-host", "NAS.example.com:443".parse().unwrap());
    assert_eq!(forwarded_host(&headers).as_deref(), Some("nas.example.com"));
}

#[test]
fn only_a_known_https_host_is_continued_to() {
    assert_eq!(
        continuation(Some("https://nas.example.com/photos?a=1"), &known(), 443),
        "https://nas.example.com/photos?a=1"
    );
    assert_eq!(
        continuation(Some("https://evil.example.net/"), &known(), 443),
        "/"
    );
    assert_eq!(
        continuation(Some("http://nas.example.com/"), &known(), 443),
        "/"
    );
    assert_eq!(continuation(Some("//evil.example.net"), &known(), 443), "/");
    assert_eq!(
        continuation(
            Some("https://nas.example.com@evil.example.net/"),
            &known(),
            443
        ),
        "/"
    );
    assert_eq!(
        continuation(Some("https://nas.example.com:8443/"), &known(), 443),
        "/"
    );
    assert_eq!(continuation(None, &known(), 443), "/");
}

#[test]
fn another_https_port_is_kept_in_both_addresses_and_required_on_return() {
    assert_eq!(
        sign_in_address(
            "https://portal.example.com:8443",
            "https://nas.example.com:8443",
            Some("/")
        ),
        "https://portal.example.com:8443/login/?return=https%3A%2F%2Fnas.example.com%3A8443%2F"
    );
    assert_eq!(
        continuation(Some("https://nas.example.com:8443/photos"), &known(), 8443),
        "https://nas.example.com:8443/photos"
    );
    assert_eq!(
        continuation(Some("https://nas.example.com/photos"), &known(), 8443),
        "/"
    );
}
