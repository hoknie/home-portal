use toml_edit::DocumentMut;

use crate::services::read_settings;

const ENVIRONMENTS: &str = "[environments.local]\nnetworks = [\"192.168.1.0/24\"]\n[environments.vpn]\nnetworks = [\"10.8.0.0/24\"]\n";

fn fields(extra: &str) -> Vec<String> {
    let document: DocumentMut = format!("{ENVIRONMENTS}{extra}").parse().unwrap();
    match read_settings(&document) {
        Ok(_) => Vec::new(),
        Err(errors) => errors.into_iter().map(|error| error.field).collect(),
    }
}

#[test]
fn a_whole_section_reads_with_its_defaults() {
    let document: DocumentMut = format!(
        "{ENVIRONMENTS}[dns]\nenabled = true\nzones = [\"Home.\"]\n[dns.addresses]\nlocal = \"192.168.1.60\"\nvpn = [\"10.8.0.1\", \"fd00::1\"]\n[[dns.records]]\nname = \"printer.home\"\ntype = \"A\"\nvalue = \"192.168.1.9\"\nenvironments = [\"local\"]\n"
    )
    .parse()
    .unwrap();
    let settings = read_settings(&document).unwrap();
    assert!(settings.enabled);
    assert_eq!(settings.port, 53);
    assert_eq!(settings.tls.port, 853);
    assert_eq!(settings.ttl, 60);
    assert_eq!(settings.zones, vec!["home"]);
    assert_eq!(settings.records[0].name, "printer.home");
    assert_eq!(settings.addresses.len(), 2);
}

#[test]
fn a_record_outside_the_zones_is_refused() {
    assert_eq!(
        fields(
            "[dns]\nzones = [\"home\"]\n[[dns.records]]\nname = \"nas.example.org\"\ntype = \"A\"\nvalue = \"1.2.3.4\"\n"
        ),
        vec!["dns.records[0].name"]
    );
}

#[test]
fn an_unknown_environment_is_refused() {
    assert_eq!(
        fields("[dns.addresses]\noffice = \"192.168.1.60\"\n"),
        vec!["dns.addresses.office"]
    );
    assert_eq!(
        fields("[dns.addresses]\ninternet = \"192.168.1.60\"\n"),
        vec!["dns.addresses.internet"]
    );
}

#[test]
fn a_ttl_outside_five_seconds_to_a_day_is_refused() {
    assert_eq!(fields("[dns]\nttl = 4\n"), vec!["dns.ttl"]);
    assert_eq!(fields("[dns]\nttl = 86401\n"), vec!["dns.ttl"]);
    assert!(fields("[dns]\nttl = 5\n").is_empty());
}

#[test]
fn the_tls_port_must_differ_from_the_plain_port() {
    assert_eq!(fields("[dns]\nport = 853\n"), vec!["dns.tls.port"]);
}

#[test]
fn dns_over_https_needs_the_proxy() {
    assert_eq!(
        fields("[dns.https]\nenabled = true\n"),
        vec!["dns.https.enabled"]
    );
    assert!(
        fields(
            "[proxy]\nenabled = true\nportal_host = \"portal.home\"\n[dns.https]\nenabled = true\n"
        )
        .is_empty()
    );
}

#[test]
fn a_certificate_needs_its_key() {
    assert_eq!(
        fields("[dns.tls]\ncertificate = \"/etc/cert.pem\"\n"),
        vec!["dns.tls.key"]
    );
    assert_eq!(
        fields("[dns.tls]\nkey = \"/etc/key.pem\"\n"),
        vec!["dns.tls.certificate"]
    );
}

#[test]
fn a_value_that_does_not_fit_its_type_and_an_unknown_type_are_refused() {
    let extra = "[dns]\nzones = [\"home\"]\n[[dns.records]]\nname = \"a.home\"\ntype = \"A\"\nvalue = \"fd00::1\"\n[[dns.records]]\nname = \"b.home\"\ntype = \"MX\"\nvalue = \"x\"\n";
    assert_eq!(
        fields(extra),
        vec!["dns.records[0].value", "dns.records[1].type"]
    );
}

#[test]
fn two_addresses_of_one_family_and_a_bad_address_are_refused() {
    assert_eq!(
        fields("[dns.addresses]\nlocal = [\"192.168.1.1\", \"192.168.1.2\"]\n"),
        vec!["dns.addresses.local"]
    );
    assert_eq!(
        fields("[dns.addresses]\nlocal = \"nas\"\n"),
        vec!["dns.addresses.local"]
    );
}

#[test]
fn a_bad_zone_address_and_port_are_named() {
    assert_eq!(
        fields("[dns]\nzones = [\"-bad\"]\naddress = \"everywhere\"\nport = 0\n"),
        vec!["dns.address", "dns.port", "dns.zones[0]"]
    );
}
