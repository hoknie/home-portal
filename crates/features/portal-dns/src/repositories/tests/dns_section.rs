use std::collections::BTreeMap;

use toml_edit::DocumentMut;

use crate::repositories::write_dns;
use crate::types::DnsChoice;

fn request() -> DnsChoice {
    DnsChoice {
        address: "0.0.0.0".into(),
        port: 5353,
        zones: vec!["home".into()],
        ttl: 60,
        addresses: BTreeMap::from([
            ("local".into(), vec!["192.168.1.60".into()]),
            ("vpn".into(), vec!["10.8.0.1".into(), "fd00::1".into()]),
        ]),
        tls: Default::default(),
        https: Default::default(),
    }
}

#[test]
fn a_new_section_is_written_whole_without_a_switch() {
    let mut document = DocumentMut::new();
    write_dns(&mut document, &request());
    assert_eq!(
        document.to_string(),
        "[dns]\naddress = \"0.0.0.0\"\nport = 5353\nzones = [\"home\"]\nttl = 60\n\n[dns.addresses]\nlocal = \"192.168.1.60\"\nvpn = [\"10.8.0.1\", \"fd00::1\"]\n\n[dns.tls]\nenabled = false\n\n[dns.https]\nenabled = false\n"
    );
}

#[test]
fn an_edit_keeps_comments_and_records() {
    let original = "# Local names.\n[dns]\nport = 53\n\n[dns.addresses]\noffice = \"10.0.0.1\"\n\n[[dns.records]]\n# The printer.\nname = \"printer.home\"\ntype = \"A\"\nvalue = \"192.168.1.9\"\n";
    let mut document: DocumentMut = original.parse().unwrap();
    write_dns(&mut document, &request());
    let text = document.to_string();
    assert!(text.starts_with("# Local names.\n[dns]\n"), "{text}");
    assert!(text.contains("port = 5353\n"), "{text}");
    assert!(!text.contains("office"), "{text}");
    assert!(
        text.contains("# The printer.\nname = \"printer.home\""),
        "{text}"
    );
}
