use axum::http::header::COOKIE;
use axum::http::{HeaderMap, HeaderValue};

use super::{
    clear_cookie, hash_password, is_argon2id, new_token, session_cookie, session_token,
    verify_password,
};
use crate::types::CookieScope;

fn scope(secure: bool, domain: Option<&str>) -> CookieScope {
    CookieScope {
        secure,
        domain: domain.map(str::to_string),
    }
}

#[test]
fn a_hashed_password_verifies_and_a_wrong_one_does_not() {
    let hash = hash_password("secret").unwrap();
    assert!(hash.starts_with("$argon2id$"));
    assert!(is_argon2id(&hash));
    assert!(verify_password("secret", Some(&hash)));
    assert!(!verify_password("wrong", Some(&hash)));
}

#[test]
fn an_unknown_user_never_verifies_even_with_the_dummy_password() {
    assert!(!verify_password(super::password::DUMMY_PASSWORD, None));
}

#[test]
fn a_hash_that_is_not_argon2id_is_recognised() {
    assert!(!is_argon2id("plain-text"));
    assert!(!is_argon2id("$2b$12$abcdefghijklmnopqrstuv"));
}

#[test]
fn a_token_is_sixty_four_hex_characters_and_never_repeats() {
    let first = new_token();
    assert_eq!(first.len(), 64);
    assert!(first.chars().all(|character| character.is_ascii_hexdigit()));
    assert_ne!(first, new_token());
}

#[test]
fn the_session_cookie_is_http_only_strict_and_secure_only_when_asked() {
    let plain = session_cookie("abc", &CookieScope::default())
        .to_str()
        .unwrap()
        .to_string();
    assert!(
        plain.contains("HttpOnly") && plain.contains("SameSite=Strict") && plain.contains("Path=/")
    );
    assert!(!plain.contains("Secure"));
    assert!(
        session_cookie("abc", &scope(true, None))
            .to_str()
            .unwrap()
            .contains("; Secure")
    );
}

#[test]
fn the_session_token_is_found_among_other_cookies() {
    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        HeaderValue::from_static("theme=dark; home_portal_session=abc123; other=1"),
    );
    assert_eq!(session_token(&headers).as_deref(), Some("abc123"));
}

#[test]
fn a_cookie_domain_is_set_and_cleared_alike() {
    let set = session_cookie("abc", &scope(true, Some("example.com")));
    assert!(
        set.to_str()
            .unwrap()
            .ends_with("; Secure; Domain=example.com")
    );
    let cleared = clear_cookie(&scope(false, Some("example.com")));
    assert!(
        cleared
            .to_str()
            .unwrap()
            .ends_with("Max-Age=0; Domain=example.com")
    );
    assert!(
        !clear_cookie(&CookieScope::default())
            .to_str()
            .unwrap()
            .contains("Domain")
    );
}

#[test]
fn a_domain_that_could_inject_attributes_is_left_out() {
    let cookie = session_cookie("abc", &scope(false, Some("example.com; Path=/evil")));
    assert!(!cookie.to_str().unwrap().contains("Domain"));
}
