use std::fs;
use std::path::Path;

use super::support::settings;
use crate::services::locate;

fn touch(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "pem").unwrap();
}

#[test]
fn explicit_files_win() {
    let found = locate(
        &settings("[dns.tls]\ncertificate = \"/etc/dns.crt\"\nkey = \"/etc/dns.key\"\n"),
        Path::new("/nowhere"),
    )
    .unwrap();
    assert_eq!(found.certificate, Path::new("/etc/dns.crt"));
    assert_eq!(found.key, Path::new("/etc/dns.key"));
}

#[test]
fn a_files_policy_gives_its_files() {
    let found = locate(
        &settings("[proxy.tls]\nmode = \"files\"\ncertificate = \"/etc/portal.crt\"\nkey = \"/etc/portal.key\"\n"),
        Path::new("/nowhere"),
    )
    .unwrap();
    assert_eq!(found.certificate, Path::new("/etc/portal.crt"));
}

#[test]
fn a_managed_caddy_gives_its_newest_certificate_for_the_host() {
    let caddy = tempfile::tempdir().unwrap();
    let local = caddy
        .path()
        .join("data/caddy/certificates/local/portal.home");
    touch(&local.join("portal.home.crt"));
    touch(&local.join("portal.home.key"));
    let found = locate(
        &settings("managed = true\n[proxy.tls]\nmode = \"internal\"\n"),
        caddy.path(),
    )
    .unwrap();
    assert_eq!(found.certificate, local.join("portal.home.crt"));
    assert!(found.modified.is_some());
}

#[test]
fn without_a_certificate_the_reason_is_given() {
    let caddy = tempfile::tempdir().unwrap();
    let waiting = locate(&settings("managed = true\n"), caddy.path()).unwrap_err();
    assert!(
        waiting.contains("no certificate for portal.home yet"),
        "{waiting}"
    );
    let unmanaged = locate(&settings(""), caddy.path()).unwrap_err();
    assert!(unmanaged.contains("dns.tls.certificate"), "{unmanaged}");
}
