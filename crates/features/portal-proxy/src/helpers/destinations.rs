use axum::http::HeaderMap;
use url::Url;
use url::form_urlencoded::byte_serialize;

pub const FORWARDED_HOST: &str = "x-forwarded-host";
pub const FORWARDED_URI: &str = "x-forwarded-uri";
pub const FORWARDED_METHOD: &str = "x-forwarded-method";
pub const LOGIN_PATH: &str = "/login/";
pub const RETURN_PARAMETER: &str = "return";
pub const HOME: &str = "/";

pub fn forwarded(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn forwarded_host(headers: &HeaderMap) -> Option<String> {
    let host = forwarded(headers, FORWARDED_HOST)?;
    let name = match host.rsplit_once(':') {
        Some((name, port)) if port.chars().all(|character| character.is_ascii_digit()) => name,
        _ => host.as_str(),
    };
    Some(name.to_ascii_lowercase())
}

pub fn sign_in_address(portal: &str, origin: &str, uri: Option<&str>) -> String {
    let path = uri.filter(|uri| uri.starts_with('/')).unwrap_or(HOME);
    let original = format!("{origin}{path}");
    let encoded: String = byte_serialize(original.as_bytes()).collect();
    format!("{portal}{LOGIN_PATH}?{RETURN_PARAMETER}={encoded}")
}

pub fn continuation(to: Option<&str>, known: &[String], https_port: u16) -> String {
    to.and_then(|to| Url::parse(to).ok())
        .filter(|url| {
            url.scheme() == "https"
                && url.username().is_empty()
                && url.password().is_none()
                && url.port_or_known_default() == Some(https_port)
                && url
                    .host_str()
                    .is_some_and(|host| known.iter().any(|known| known == host))
        })
        .map_or_else(|| HOME.to_string(), |url| url.to_string())
}
