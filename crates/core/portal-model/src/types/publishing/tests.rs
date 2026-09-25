use super::{Publication, TlsMode, TlsPolicy};
use crate::types::Environment;

#[test]
fn a_host_name_is_lower_case_labels_separated_by_dots() {
    assert_eq!(Publication::host_problem("media.example.com"), None);
    assert_eq!(Publication::host_problem("nas.home.arpa"), None);
    assert_eq!(Publication::host_problem("portal"), None);
}

#[test]
fn a_host_name_carries_no_scheme_port_or_path() {
    assert!(Publication::host_problem("https://x").is_some());
    assert!(Publication::host_problem("x:443").is_some());
    assert!(Publication::host_problem("x.example.com/path").is_some());
}

#[test]
fn a_host_label_does_not_start_or_end_with_a_hyphen() {
    assert!(Publication::host_problem("-a.example.com").is_some());
    assert!(Publication::host_problem("a-.example.com").is_some());
    assert!(Publication::host_problem("a..example.com").is_some());
}

#[test]
fn an_empty_or_upper_case_host_is_refused() {
    assert!(Publication::host_problem("").is_some());
    assert!(Publication::host_problem("Media.example.com").is_some());
}

#[test]
fn a_publication_defaults_to_internet_without_sign_in() {
    let publication: Publication = serde_json::from_str(r#"{"host":"media.example.com"}"#).unwrap();
    assert_eq!(publication, Publication::new("media.example.com"));
    assert!(publication.published_in(&Environment::internet()));
    assert!(publication.auth.is_empty());
    assert!(publication.upstream_verify);
    assert_eq!(publication.address_on(443), "https://media.example.com");
    assert_eq!(
        publication.address_on(8443),
        "https://media.example.com:8443"
    );
}

#[test]
fn a_host_is_within_its_own_domain_and_its_parents_only() {
    assert!(Publication::is_within("nas.example.com", "example.com"));
    assert!(Publication::is_within("example.com", "example.com"));
    assert!(!Publication::is_within("nasexample.com", "example.com"));
    assert!(!Publication::is_within("nas.example.org", "example.com"));
}

#[test]
fn files_mode_needs_a_certificate_and_a_key() {
    let policy = TlsPolicy {
        mode: TlsMode::Files,
        ..TlsPolicy::default()
    };
    let fields: Vec<&str> = policy
        .problems()
        .into_iter()
        .map(|(field, _)| field)
        .collect();
    assert_eq!(fields, vec!["certificate", "key"]);
    assert!(TlsPolicy::default().problems().is_empty());
}

#[test]
fn an_acme_email_must_look_like_an_address() {
    let policy = TlsPolicy {
        email: Some("nobody".to_string()),
        ..TlsPolicy::default()
    };
    assert_eq!(policy.problems(), vec![("email", TlsPolicy::INVALID_EMAIL)]);
}
