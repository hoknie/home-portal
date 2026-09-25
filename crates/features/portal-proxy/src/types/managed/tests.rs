use super::{CaddySource, CaddyVersion};

#[test]
fn the_default_source_asks_github_for_the_latest_release() {
    assert_eq!(
        CaddySource::default().release_url(),
        "https://api.github.com/repos/caddyserver/caddy/releases/latest"
    );
}

#[test]
fn an_exact_version_asks_for_its_tag() {
    let source = CaddySource {
        version: CaddyVersion::parse("2.10.2").unwrap(),
        ..CaddySource::default()
    };
    assert_eq!(
        source.release_url(),
        "https://api.github.com/repos/caddyserver/caddy/releases/tags/v2.10.2"
    );
}

#[test]
fn a_leading_v_is_dropped() {
    assert_eq!(
        CaddyVersion::parse("v2.10.2"),
        Ok(CaddyVersion::Exact("2.10.2".to_string()))
    );
}

#[test]
fn a_pre_release_is_an_exact_version() {
    assert_eq!(
        CaddyVersion::parse("2.11.0-beta.1"),
        Ok(CaddyVersion::Exact("2.11.0-beta.1".to_string()))
    );
}

#[test]
fn a_version_that_is_not_three_numbers_is_refused() {
    for text in ["newest", "2.10", "2.10.x", "2.10.2-", ""] {
        assert_eq!(
            CaddyVersion::parse(text),
            Err(CaddyVersion::PROBLEM),
            "{text}"
        );
    }
}

#[test]
fn a_trailing_slash_on_the_base_is_ignored() {
    assert_eq!(
        CaddySource::parse_base("https://git.example.com/api/v1/repos/caddy/caddy/releases/"),
        Ok("https://git.example.com/api/v1/repos/caddy/caddy/releases".to_string())
    );
}

#[test]
fn plain_http_is_allowed_only_on_loopback() {
    assert!(CaddySource::parse_base("http://127.0.0.1:8080/releases").is_ok());
    assert!(CaddySource::parse_base("http://localhost/releases").is_ok());
    assert!(CaddySource::parse_base("http://mirror.example.com/releases").is_err());
    assert!(CaddySource::parse_base("ftp://mirror.example.com/releases").is_err());
    assert!(CaddySource::parse_base("not a url").is_err());
}
