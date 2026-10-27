use std::collections::BTreeMap;

use toml_edit::DocumentMut;

use crate::repositories::write_dns;
use crate::types::{DnsChoice, DnsHttpsChoice, DnsTlsChoice};

const COMMENTED: &str = "# Local names.\n[dns]\naddress = \"0.0.0.0\" # all\nport = 53\n\n[dns.tls]\nenabled = true\ncertificate = \"/old.pem\" # the old one\n\n[dns.https]\nenabled = true\nhost = \"dns.home\"\n";

fn spaced() -> DnsChoice {
    DnsChoice {
        address: "  0.0.0.0 ".into(),
        port: 5353,
        zones: vec![" home ".into(), "lan".into()],
        ttl: 60,
        addresses: BTreeMap::from([
            ("empty".into(), Vec::new()),
            ("local".into(), vec![" 192.168.1.60 ".into()]),
            ("vpn".into(), vec![" 10.8.0.1".into(), "fd00::1 ".into()]),
        ]),
        tls: DnsTlsChoice {
            enabled: true,
            port: Some(853),
            certificate: Some("   ".into()),
            key: Some(" /key.pem ".into()),
        },
        https: DnsHttpsChoice {
            enabled: true,
            host: Some(" ".into()),
        },
    }
}

fn stored(text: &str, choice: &DnsChoice) -> String {
    let mut document: DocumentMut = text.parse().unwrap();
    write_dns(&mut document, &choice.normalized());
    document.to_string()
}

fn cases() -> Vec<(&'static str, String)> {
    vec![
        ("spaced_into_empty", stored("", &spaced())),
        ("spaced_into_commented", stored(COMMENTED, &spaced())),
        (
            "blank_texts_removed",
            stored(
                COMMENTED,
                &DnsChoice {
                    tls: DnsTlsChoice {
                        certificate: Some(String::new()),
                        ..DnsTlsChoice::default()
                    },
                    https: DnsHttpsChoice {
                        enabled: false,
                        host: None,
                    },
                    ..spaced()
                },
            ),
        ),
    ]
}

const EXPECTED: [(&str, &str); 3] = [
    (
        "spaced_into_empty",
        r#"[dns]
address = "0.0.0.0"
port = 5353
zones = ["home", "lan"]
ttl = 60

[dns.addresses]
empty = []
local = "192.168.1.60"
vpn = ["10.8.0.1", "fd00::1"]

[dns.tls]
enabled = true
port = 853
key = "/key.pem"

[dns.https]
enabled = true
"#,
    ),
    (
        "spaced_into_commented",
        r#"# Local names.
[dns]
address = "0.0.0.0" # all
port = 5353
zones = ["home", "lan"]
ttl = 60

[dns.tls]
enabled = true
port = 853
key = "/key.pem"

[dns.https]
enabled = true

[dns.addresses]
empty = []
local = "192.168.1.60"
vpn = ["10.8.0.1", "fd00::1"]
"#,
    ),
    (
        "blank_texts_removed",
        r#"# Local names.
[dns]
address = "0.0.0.0" # all
port = 5353
zones = ["home", "lan"]
ttl = 60

[dns.tls]
enabled = false

[dns.https]
enabled = false

[dns.addresses]
empty = []
local = "192.168.1.60"
vpn = ["10.8.0.1", "fd00::1"]
"#,
    ),
];

#[test]
fn dns_writes_keep_their_bytes() {
    let cases = cases();
    assert_eq!(cases.len(), EXPECTED.len());
    for ((name, text), (expected_name, expected)) in cases.iter().zip(EXPECTED) {
        assert_eq!(*name, expected_name);
        assert_eq!(text, expected, "{name}");
    }
}
