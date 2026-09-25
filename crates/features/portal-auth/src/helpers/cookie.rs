use axum::http::header::COOKIE;
use axum::http::{HeaderMap, HeaderValue};

use crate::types::CookieScope;

pub const SESSION_COOKIE: &str = "home_portal_session";
pub const SESSION_SECONDS: i64 = 7 * 24 * 60 * 60;

pub fn session_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == SESSION_COOKIE)
        .map(|(_, token)| token.to_string())
}

pub fn session_cookie(token: &str, scope: &CookieScope) -> HeaderValue {
    let secure = if scope.secure { "; Secure" } else { "" };
    let domain = domain_attribute(scope.domain.as_deref());
    let cookie = format!(
        "{SESSION_COOKIE}={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age={SESSION_SECONDS}{secure}{domain}"
    );
    HeaderValue::from_str(&cookie).expect("a hex token and a host name make a valid cookie")
}

pub fn clear_cookie(scope: &CookieScope) -> HeaderValue {
    let secure = if scope.secure { "; Secure" } else { "" };
    let domain = domain_attribute(scope.domain.as_deref());
    HeaderValue::from_str(&format!(
        "{SESSION_COOKIE}=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0{secure}{domain}"
    ))
    .expect("a host name makes a valid cookie")
}

fn domain_attribute(domain: Option<&str>) -> String {
    domain
        .filter(|domain| {
            domain.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '-')
            })
        })
        .map(|domain| format!("; Domain={domain}"))
        .unwrap_or_default()
}
