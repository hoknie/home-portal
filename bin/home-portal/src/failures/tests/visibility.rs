use axum::http::{HeaderMap, HeaderValue};

use super::support::{HOST, LOCAL, OUTSIDE, PHONE, forwarded_for, main_of, peer};
use crate::failures::main_file::MainFile;
use crate::failures::visibility::sees_details;

#[test]
fn the_host_itself_sees_the_details() {
    assert!(sees_details(
        peer(HOST),
        &HeaderMap::new(),
        &MainFile::defaults()
    ));
    assert!(sees_details(
        peer("[::1]:5000"),
        &HeaderMap::new(),
        &MainFile::defaults()
    ));
}

#[test]
fn the_home_network_sees_the_details_when_the_main_file_names_it() {
    assert!(sees_details(
        peer(PHONE),
        &HeaderMap::new(),
        &main_of(LOCAL)
    ));
}

#[test]
fn an_address_outside_every_environment_does_not_see_the_details() {
    assert!(!sees_details(
        peer(OUTSIDE),
        &HeaderMap::new(),
        &main_of(LOCAL)
    ));
}

#[test]
fn without_readable_environments_only_the_host_sees_the_details() {
    assert!(!sees_details(
        peer(PHONE),
        &HeaderMap::new(),
        &MainFile::defaults()
    ));
}

#[test]
fn a_request_forwarded_by_a_proxy_that_is_not_trusted_counts_as_outside() {
    let headers = forwarded_for("203.0.113.9");
    assert!(!sees_details(peer(HOST), &headers, &MainFile::defaults()));
    let mut real = HeaderMap::new();
    real.insert("x-real-ip", HeaderValue::from_static("192.168.1.40"));
    assert!(!sees_details(peer(HOST), &real, &main_of(LOCAL)));
}

#[test]
fn a_trusted_proxy_is_judged_by_the_visitor_it_forwards() {
    let main = main_of(&format!(
        "[network]\ntrusted_proxies = [\"127.0.0.1/32\"]\n\n{LOCAL}"
    ));
    assert!(!sees_details(
        peer(HOST),
        &forwarded_for("203.0.113.9"),
        &main
    ));
    assert!(sees_details(
        peer(HOST),
        &forwarded_for("192.168.1.40"),
        &main
    ));
}

#[test]
fn a_chosen_environment_cookie_does_not_count() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "cookie",
        HeaderValue::from_static("portal_environment=local"),
    );
    assert!(!sees_details(peer(OUTSIDE), &headers, &main_of(LOCAL)));
}
