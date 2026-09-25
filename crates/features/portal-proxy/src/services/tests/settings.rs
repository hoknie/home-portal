use portal_feature::FieldError;
use toml_edit::DocumentMut;

use crate::services::publications::{
    NO_COOKIE_DOMAIN, OUTSIDE_COOKIE_DOMAIN, PORTAL_HOST, TAKEN_HOST,
};
use crate::services::settings::{LOOPBACK_NOT_TRUSTED, NOT_A_PARENT, PUBLIC_URL_DIFFERS, REQUIRED};
use crate::services::{read_settings, validate_publications, validate_settings};
use crate::types::AdminAddress;

const ENABLED: &str = "[network]\ntrusted_proxies = [\"127.0.0.1\"]\n\n[proxy]\nenabled = true\nportal_host = \"portal.example.com\"\ncookie_domain = \"example.com\"\n";

fn document(text: &str) -> DocumentMut {
    text.parse().unwrap()
}

fn fields(errors: &[FieldError]) -> Vec<&str> {
    errors.iter().map(|error| error.field.as_str()).collect()
}

fn with_service(proxy: &str) -> DocumentMut {
    document(&format!(
        "{ENABLED}\n[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://192.168.1.5\"\nproxy = {proxy}\n"
    ))
}

#[test]
fn without_a_proxy_section_the_proxy_is_disabled() {
    let settings = read_settings(&document("")).unwrap();
    assert!(!settings.enabled);
    assert_eq!(settings.admin, AdminAddress::default());
    assert_eq!(settings.admin.to_string(), AdminAddress::DEFAULT);
}

#[test]
fn an_enabled_proxy_with_loopback_trusted_is_accepted() {
    let settings = read_settings(&document(ENABLED)).unwrap();
    assert!(settings.enabled);
    assert_eq!(settings.active(), Some("portal.example.com"));
    assert_eq!(settings.cookie_domain(), Some("example.com"));
}

#[test]
fn a_remote_admin_address_is_refused() {
    let errors = validate_settings(&document("[proxy]\nadmin = \"http://192.168.1.5:2019\"\n"));
    assert_eq!(fields(&errors), vec!["proxy.admin"]);
}

#[test]
fn an_admin_address_may_be_localhost_or_an_absolute_unix_socket() {
    assert!(AdminAddress::parse("http://localhost:2019").is_ok());
    assert!(AdminAddress::parse("http://[::1]:2019").is_ok());
    assert!(AdminAddress::parse("unix:/run/caddy/admin.sock").is_ok());
    assert!(AdminAddress::parse("unix:admin.sock").is_err());
    assert!(AdminAddress::parse("https://127.0.0.1:2019").is_err());
}

#[test]
fn loopback_must_be_trusted_when_the_proxy_is_enabled() {
    let errors = validate_settings(&document(
        "[proxy]\nenabled = true\nportal_host = \"portal.example.com\"\n",
    ));
    assert_eq!(fields(&errors), vec!["network.trusted_proxies"]);
    assert_eq!(errors[0].message, LOOPBACK_NOT_TRUSTED);
}

#[test]
fn a_range_covering_loopback_is_trusted_enough() {
    let errors = validate_settings(&document(
        "[network]\ntrusted_proxies = [\"127.0.0.0/8\"]\n\n[proxy]\nenabled = true\nportal_host = \"portal.example.com\"\n",
    ));
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn an_enabled_proxy_needs_the_portal_host() {
    let errors = validate_settings(&document(
        "[network]\ntrusted_proxies = [\"127.0.0.1\"]\n\n[proxy]\nenabled = true\n",
    ));
    assert_eq!(fields(&errors), vec!["proxy.portal_host"]);
    assert_eq!(errors[0].message, REQUIRED);
}

#[test]
fn a_cookie_domain_outside_the_portal_host_is_refused() {
    let errors = validate_settings(&document(
        "[proxy]\nportal_host = \"portal.example.com\"\ncookie_domain = \"example.org\"\n",
    ));
    assert_eq!(fields(&errors), vec!["proxy.cookie_domain"]);
    assert_eq!(errors[0].message, NOT_A_PARENT);
}

#[test]
fn the_public_url_must_be_the_portal_host_over_https() {
    let text = ENABLED.replace(
        "[network]\n",
        "[network]\npublic_url = \"https://home.example.com\"\n",
    );
    let errors = validate_settings(&document(&text));
    assert_eq!(fields(&errors), vec!["network.public_url"]);
    assert_eq!(errors[0].message, PUBLIC_URL_DIFFERS);
    let matching = ENABLED.replace(
        "[network]\n",
        "[network]\npublic_url = \"https://portal.example.com/\"\n",
    );
    assert!(validate_settings(&document(&matching)).is_empty());
}

#[test]
fn files_mode_without_a_key_is_refused_by_name() {
    let errors = validate_settings(&document(
        "[proxy.tls]\nmode = \"files\"\ncertificate = \"/etc/ssl/portal.pem\"\n",
    ));
    assert_eq!(fields(&errors), vec!["proxy.tls.key"]);
}

#[test]
fn two_services_may_not_publish_one_host() {
    let text = format!(
        "{ENABLED}\n[[services]]\nid = \"a\"\nname = \"A\"\nurl = \"http://a\"\nproxy = {{ host = \"media.example.com\" }}\n\n[[services]]\nid = \"b\"\nname = \"B\"\nurl = \"http://b\"\nproxy = {{ host = \"media.example.com\" }}\n"
    );
    let errors = validate_publications(&document(&text));
    assert_eq!(fields(&errors), vec!["services[1].proxy.host"]);
    assert_eq!(errors[0].message, TAKEN_HOST);
}

#[test]
fn a_service_may_not_publish_the_portal_host() {
    let errors = validate_publications(&with_service("{ host = \"portal.example.com\" }"));
    assert_eq!(fields(&errors), vec!["services[0].proxy.host"]);
    assert_eq!(errors[0].message, PORTAL_HOST);
}

#[test]
fn sign_in_outside_the_cookie_domain_is_refused() {
    let errors = validate_publications(&with_service(
        "{ host = \"nas.example.org\", auth = [\"internet\"] }",
    ));
    assert_eq!(fields(&errors), vec!["services[0].proxy.auth"]);
    assert_eq!(errors[0].message, OUTSIDE_COOKIE_DOMAIN);
}

#[test]
fn sign_in_without_a_cookie_domain_is_refused() {
    let text = format!(
        "{}\n[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://192.168.1.5\"\nproxy = {{ host = \"nas.example.com\", auth = [\"internet\"] }}\n",
        ENABLED.replace("cookie_domain = \"example.com\"\n", "")
    );
    let errors = validate_publications(&document(&text));
    assert_eq!(fields(&errors), vec!["services[0].proxy.auth"]);
    assert_eq!(errors[0].message, NO_COOKIE_DOMAIN);
}

#[test]
fn sign_in_under_the_cookie_domain_is_accepted() {
    let errors = validate_publications(&with_service(
        "{ host = \"nas.example.com\", auth = [\"internet\"] }",
    ));
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn a_publication_is_accepted_while_the_proxy_is_disabled() {
    let text = "[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://192.168.1.5\"\nproxy = { host = \"nas.example.org\", auth = [\"internet\"] }\n";
    assert!(validate_publications(&document(text)).is_empty());
    assert!(validate_settings(&document(text)).is_empty());
}

#[test]
fn a_plain_http_mirror_is_refused() {
    let errors = validate_settings(&document(
        "[proxy.caddy]\nsource = \"http://mirror.example.com/caddy/releases\"\n",
    ));
    assert_eq!(fields(&errors), vec!["proxy.caddy.source"]);
}

#[test]
fn a_loopback_mirror_over_http_is_accepted() {
    let settings = read_settings(&document(
        "[proxy.caddy]\nsource = \"http://127.0.0.1:8080/releases/\"\n",
    ))
    .unwrap();
    assert_eq!(settings.caddy.base, "http://127.0.0.1:8080/releases");
}

#[test]
fn a_malformed_caddy_version_is_refused_even_while_the_proxy_is_off() {
    let errors = validate_settings(&document("[proxy]\ncaddy = { version = \"newest\" }\n"));
    assert_eq!(fields(&errors), vec!["proxy.caddy.version"]);
}

#[test]
fn a_pre_release_caddy_version_is_accepted() {
    let settings = read_settings(&document(
        "[proxy]\ncaddy = { version = \"v2.11.0-beta.1\" }\n",
    ))
    .unwrap();
    assert_eq!(settings.caddy.version.to_string(), "2.11.0-beta.1");
}
