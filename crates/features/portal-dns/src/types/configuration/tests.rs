use std::collections::BTreeMap;

use super::{DnsChoice, DnsHttpsChoice, DnsTlsChoice};

#[test]
fn normalizing_trims_every_text_and_drops_blank_optional_ones() {
    let choice = DnsChoice {
        address: " 0.0.0.0 ".into(),
        zones: vec![" home ".into()],
        addresses: BTreeMap::from([(" vpn ".into(), vec![" 10.8.0.1".into()])]),
        tls: DnsTlsChoice {
            enabled: true,
            port: Some(853),
            certificate: Some("  ".into()),
            key: Some(" /key.pem ".into()),
        },
        https: DnsHttpsChoice {
            enabled: true,
            host: Some(String::new()),
        },
        ..DnsChoice::default()
    };
    let normalized = choice.normalized();
    assert_eq!(normalized.address, "0.0.0.0");
    assert_eq!(normalized.zones, ["home"]);
    assert_eq!(normalized.addresses[" vpn "], ["10.8.0.1"]);
    assert_eq!(normalized.tls.certificate, None);
    assert_eq!(normalized.tls.key.as_deref(), Some("/key.pem"));
    assert_eq!(normalized.tls.port, Some(853));
    assert_eq!(normalized.https.host, None);
}
