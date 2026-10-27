use portal_model::{TlsMode, TlsPolicy};
use toml_edit::DocumentMut;

use portal_feature::ModulePreparer;

use crate::repositories::write_choice;
use crate::types::ProxyChoice;
use crate::usecases::PrepareProxy;

const COMMENTED: &str = "# mine\n[proxy]\nmanaged = false # not yet\ntls = { mode = \"files\", certificate = \"/old.pem\", key = \"/old.key\" }\n";

const NETWORK: &str = "# where the portal listens\n[network]\nport = 8080\ntrusted_proxies = [\"10.0.0.0/8\"] # the vpn\n";

const COVERED: &str = "[network]\ntrusted_proxies = [\"127.0.0.1/32\", \"10.0.0.0/8\"]\n";

fn spaced() -> ProxyChoice {
    ProxyChoice {
        http_port: Some(80),
        https_port: Some(8443),
        portal_host: Some("portal.example.com".into()),
        cookie_domain: Some(" example.com ".into()),
        tls: TlsPolicy {
            mode: TlsMode::Files,
            email: Some("   ".into()),
            certificate: Some(" /new.pem ".into()),
            key: Some(String::new()),
        },
    }
}

fn chosen(text: &str, choice: &ProxyChoice) -> String {
    let mut document: DocumentMut = text.parse().unwrap();
    write_choice(&mut document, &choice.normalized());
    document.to_string()
}

fn prepared(text: &str) -> String {
    let mut document: DocumentMut = text.parse().unwrap();
    PrepareProxy.prepare(&mut document);
    document.to_string()
}

fn cases() -> Vec<(&'static str, String)> {
    vec![
        ("choice_into_empty", chosen("", &spaced())),
        ("choice_into_commented", chosen(COMMENTED, &spaced())),
        (
            "loopback_without_network",
            prepared("[proxy]\nmanaged = true\n"),
        ),
        ("loopback_beside_a_network", prepared(NETWORK)),
        ("loopback_already_covered", prepared(COVERED)),
    ]
}

const EXPECTED: [(&str, &str); 5] = [
    (
        "choice_into_empty",
        r#"[proxy]
https_port = 8443
portal_host = "portal.example.com"
cookie_domain = " example.com "
tls = { mode = "files", certificate = " /new.pem " }
"#,
    ),
    (
        "choice_into_commented",
        r#"# mine
[proxy]
managed = false # not yet
tls = { mode = "files", certificate = " /new.pem " }
https_port = 8443
portal_host = "portal.example.com"
cookie_domain = " example.com "
"#,
    ),
    (
        "loopback_without_network",
        r#"[proxy]
managed = true

[network]
trusted_proxies = ["127.0.0.1"]
"#,
    ),
    (
        "loopback_beside_a_network",
        r#"# where the portal listens
[network]
port = 8080
trusted_proxies = ["10.0.0.0/8", "127.0.0.1"] # the vpn
"#,
    ),
    (
        "loopback_already_covered",
        r#"[network]
trusted_proxies = ["127.0.0.1/32", "10.0.0.0/8"]
"#,
    ),
];

#[test]
fn proxy_writes_keep_their_bytes() {
    let cases = cases();
    assert_eq!(cases.len(), EXPECTED.len());
    for ((name, text), (expected_name, expected)) in cases.iter().zip(EXPECTED) {
        assert_eq!(*name, expected_name);
        assert_eq!(text, expected, "{name}");
    }
}
