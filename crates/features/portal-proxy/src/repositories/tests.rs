use toml_edit::DocumentMut;

use super::write_managed;

const BEFORE: &str = "# the proxy\n[proxy]\nadmin = \"http://127.0.0.1:2019\" # on\nportal_host = \"portal.example.com\"\n";

#[test]
fn starting_caddy_adds_managed_and_keeps_every_comment() {
    let mut document: DocumentMut = BEFORE.parse().unwrap();
    write_managed(&mut document, true);
    assert_eq!(document.to_string(), format!("{BEFORE}managed = true\n"));
    write_managed(&mut document, false);
    assert_eq!(document.to_string(), format!("{BEFORE}managed = false\n"));
}

#[test]
fn a_file_without_the_section_gains_it() {
    let mut document: DocumentMut = "[network]\nport = 8080\n".parse().unwrap();
    write_managed(&mut document, true);
    assert_eq!(
        document.to_string(),
        "[network]\nport = 8080\n\n[proxy]\nmanaged = true\n"
    );
}

fn choice() -> crate::types::ProxyChoice {
    crate::types::ProxyChoice {
        http_port: None,
        https_port: None,
        portal_host: Some("portal.example.com".into()),
        cookie_domain: None,
        tls: portal_model::TlsPolicy {
            email: Some("owner@example.com".into()),
            ..portal_model::TlsPolicy::default()
        },
    }
}

#[test]
fn writing_the_settings_leaves_other_keys_as_they_were_and_keeps_every_comment() {
    let mut document: DocumentMut =
        "# mine\n[proxy]\nmanaged = false # not yet\ncookie_domain = \"old.example.com\"\n"
            .parse()
            .unwrap();
    super::write_choice(&mut document, &choice());
    assert_eq!(
        document.to_string(),
        "# mine\n[proxy]\nmanaged = false # not yet\nportal_host = \"portal.example.com\"\ntls = { mode = \"acme\", email = \"owner@example.com\" }\n"
    );
}

#[test]
fn a_tls_table_is_edited_in_place() {
    let mut document: DocumentMut = "[proxy]\nmanaged = true\n\n# certificates\n[proxy.tls]\nmode = \"files\"\ncertificate = \"/a.pem\"\nkey = \"/a.key\"\n".parse().unwrap();
    super::write_choice(&mut document, &choice());
    assert_eq!(
        document.to_string(),
        "[proxy]\nmanaged = true\nportal_host = \"portal.example.com\"\n\n# certificates\n[proxy.tls]\nmode = \"acme\"\nemail = \"owner@example.com\"\n"
    );
}

#[test]
fn writing_the_settings_never_adds_a_switch() {
    let mut document = DocumentMut::new();
    super::write_choice(&mut document, &choice());
    assert!(!document.to_string().contains("enabled"), "{document}");
}

#[test]
fn preparing_the_proxy_trusts_loopback_and_touches_only_the_network_section() {
    use portal_feature::{Module, ModulePreparer};
    let preparer = crate::usecases::PrepareProxy;
    assert_eq!(preparer.module(), Module::Proxy);
    assert_eq!(preparer.touches(), &["network"]);
    let mut document = DocumentMut::new();
    preparer.prepare(&mut document);
    assert_eq!(
        document.to_string(),
        "[network]\ntrusted_proxies = [\"127.0.0.1\"]\n"
    );
}

#[test]
fn loopback_is_trusted_once() {
    let mut document: DocumentMut = "[network]\ntrusted_proxies = [\"10.0.0.1\"]\n"
        .parse()
        .unwrap();
    super::trust_loopback(&mut document);
    super::trust_loopback(&mut document);
    assert_eq!(
        document.to_string(),
        "[network]\ntrusted_proxies = [\"10.0.0.1\", \"127.0.0.1\"]\n"
    );
    let mut covered: DocumentMut = "[network]\ntrusted_proxies = [\"127.0.0.0/8\"]\n"
        .parse()
        .unwrap();
    super::trust_loopback(&mut covered);
    assert_eq!(
        covered.to_string(),
        "[network]\ntrusted_proxies = [\"127.0.0.0/8\"]\n"
    );
    let mut empty = DocumentMut::new();
    super::trust_loopback(&mut empty);
    assert_eq!(
        empty.to_string(),
        "[network]\ntrusted_proxies = [\"127.0.0.1\"]\n"
    );
}

fn pinned(version: &str) -> crate::types::CaddySource {
    crate::types::CaddySource {
        version: crate::types::CaddyVersion::parse(version).unwrap(),
        ..crate::types::CaddySource::default()
    }
}

#[test]
fn pinning_a_caddy_version_writes_an_inline_table_and_keeps_every_comment() {
    let mut document: DocumentMut = BEFORE.parse().unwrap();
    super::write_caddy_source(&mut document, &pinned("2.10.2"));
    assert_eq!(
        document.to_string(),
        format!("{BEFORE}caddy = {{ version = \"2.10.2\" }}\n")
    );
}

#[test]
fn the_default_source_and_version_remove_the_caddy_key() {
    let mut document: DocumentMut = format!("{BEFORE}caddy = {{ version = \"2.10.2\" }}\n")
        .parse()
        .unwrap();
    super::write_caddy_source(&mut document, &crate::types::CaddySource::default());
    assert_eq!(document.to_string(), BEFORE);
    let mut empty = DocumentMut::new();
    super::write_caddy_source(&mut empty, &crate::types::CaddySource::default());
    assert_eq!(empty.to_string(), "");
}

#[test]
fn a_caddy_table_is_edited_in_place() {
    let mut document: DocumentMut = "[proxy]\nmanaged = true\n\n# where caddy comes from\n[proxy.caddy]\nsource = \"https://git.example.com/releases\" # mirror\nversion = \"2.9.0\"\n".parse().unwrap();
    let source = crate::types::CaddySource {
        base: "https://git.example.com/releases".into(),
        ..pinned("2.10.2")
    };
    super::write_caddy_source(&mut document, &source);
    assert_eq!(
        document.to_string(),
        "[proxy]\nmanaged = true\n\n# where caddy comes from\n[proxy.caddy]\nsource = \"https://git.example.com/releases\" # mirror\nversion = \"2.10.2\"\n"
    );
}

#[test]
fn editing_the_settings_keeps_the_download_source() {
    let mirror =
        "caddy = { source = \"https://git.example.com/api/v1/repos/caddy/caddy/releases\" }\n";
    let mut document: DocumentMut = format!("[proxy]\nmanaged = false\n{mirror}")
        .parse()
        .unwrap();
    super::write_choice(&mut document, &choice());
    assert!(document.to_string().contains(mirror), "{document}");
}
