use std::collections::BTreeMap;

use portal_feature::FieldError;
use portal_model::{Environment, Environments, Publication, TlsMode, TlsPolicy};

use super::check_publication;
use super::publication_validation::{
    GIVEN_BY_PROXY, MISSING_UPSTREAM, NOT_SHOWN, UNKNOWN_ENVIRONMENT,
};
use crate::types::{ProbeKind, ServiceEntry};

fn known() -> Environments {
    Environments::new(vec![(Environment::parse("local").unwrap(), Vec::new())])
}

fn published(host: &str) -> ServiceEntry {
    let mut entry = ServiceEntry::new("media", "Media", "http://192.168.1.10:8096");
    entry.proxy = Some(Publication::new(host));
    entry
}

fn errors_of(entry: &ServiceEntry) -> Vec<FieldError> {
    check_publication(entry, &known())
}

fn fields(errors: &[FieldError]) -> Vec<&str> {
    errors.iter().map(|error| error.field.as_str()).collect()
}

#[test]
fn a_published_service_with_defaults_is_accepted() {
    assert!(errors_of(&published("media.example.com")).is_empty());
}

#[test]
fn a_service_without_publication_is_not_checked() {
    assert!(errors_of(&ServiceEntry::new("tcp", "Tcp", "tcp://nas:22")).is_empty());
}

#[test]
fn an_address_for_a_published_environment_is_refused() {
    let mut entry = published("media.example.com");
    entry.addresses = BTreeMap::from([(
        "internet".to_string(),
        "https://old.example.com".to_string(),
    )]);
    let errors = errors_of(&entry);
    assert_eq!(fields(&errors), vec!["addresses.internet"]);
    assert_eq!(errors[0].message, GIVEN_BY_PROXY);
}

#[test]
fn publishing_where_the_service_is_hidden_is_refused() {
    let mut entry = published("media.example.com");
    entry.environments = Some(vec!["local".to_string()]);
    let errors = errors_of(&entry);
    assert_eq!(fields(&errors), vec!["proxy.environments[0]"]);
    assert_eq!(errors[0].message, NOT_SHOWN);
}

#[test]
fn a_tcp_service_without_an_upstream_is_refused() {
    let mut entry = ServiceEntry::new("nas", "NAS", "tcp://nas.home.lan:22");
    entry.probe.kind = ProbeKind::Tcp;
    entry.proxy = Some(Publication::new("nas.example.com"));
    let errors = errors_of(&entry);
    assert_eq!(fields(&errors), vec!["proxy.upstream"]);
    assert_eq!(errors[0].message, MISSING_UPSTREAM);
    entry.proxy.as_mut().unwrap().upstream = Some("http://nas.home.lan:5000".to_string());
    assert!(errors_of(&entry).is_empty());
}

#[test]
fn unknown_environments_are_refused_in_publication_and_sign_in() {
    let mut entry = published("media.example.com");
    let publication = entry.proxy.as_mut().unwrap();
    publication.environments = vec!["office".to_string()];
    publication.auth = vec!["vpn".to_string()];
    let errors = errors_of(&entry);
    assert_eq!(
        fields(&errors),
        vec!["proxy.environments[0]", "proxy.auth[0]"]
    );
    assert!(
        errors
            .iter()
            .all(|error| error.message == UNKNOWN_ENVIRONMENT)
    );
}

#[test]
fn a_bad_host_upstream_and_tls_override_are_named() {
    let mut entry = published("https://media");
    let publication = entry.proxy.as_mut().unwrap();
    publication.upstream = Some("ftp://media".to_string());
    publication.tls = Some(TlsPolicy {
        mode: TlsMode::Files,
        ..TlsPolicy::default()
    });
    assert_eq!(
        fields(&errors_of(&entry)),
        vec![
            "proxy.host",
            "proxy.upstream",
            "proxy.tls.certificate",
            "proxy.tls.key"
        ]
    );
}

#[test]
fn a_published_address_carries_a_port_other_than_443() {
    let entry = published("media.example.com");
    assert_eq!(
        entry.shown_address(&Environment::internet(), Some(8443)),
        "https://media.example.com:8443"
    );
    assert_eq!(
        entry.shown_address(&Environment::internet(), None),
        "http://192.168.1.10:8096"
    );
}
