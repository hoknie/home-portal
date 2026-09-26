use std::collections::BTreeMap;

use portal_dns::{
    DnsHttpsSettingsResponse, DnsResponse, DnsSettingsResponse, DnsTlsSettingsResponse,
    EnvironmentAnswerResponse, NameResponse, RecordResponse, TransportResponse, ZoneResponse,
};

use crate::check;

fn record(kind: &str, value: &str) -> RecordResponse {
    RecordResponse {
        kind: kind.into(),
        value: value.into(),
    }
}

fn answer(environment: &str, records: Vec<RecordResponse>) -> EnvironmentAnswerResponse {
    EnvironmentAnswerResponse {
        environment: environment.into(),
        records,
    }
}

#[test]
fn the_dns_sample_matches_its_serializer() {
    let serial = 2_718_281_828;
    let dns = DnsResponse {
        enabled: true,
        plain: TransportResponse {
            listening: false,
            address: Some("0.0.0.0:53".into()),
            reason: Some("Address already in use (os error 48) (0.0.0.0:53)".into()),
        },
        tls: TransportResponse {
            listening: true,
            address: Some("0.0.0.0:853".into()),
            reason: None,
        },
        https: TransportResponse {
            listening: true,
            address: None,
            reason: None,
        },
        last_error: Some("Address already in use (os error 48) (0.0.0.0:53)".into()),
        zones: vec![ZoneResponse {
            apex: "home".into(),
            single: false,
            serial,
        }],
        names: vec![
            NameResponse {
                name: "jellyfin.home".into(),
                answers: vec![
                    answer("local", vec![record("A", "192.168.1.60")]),
                    answer("vpn", vec![record("A", "10.8.0.1")]),
                ],
            },
            NameResponse {
                name: "nas.home".into(),
                answers: vec![answer("local", vec![record("A", "192.168.1.60")])],
            },
            NameResponse {
                name: "portal.home".into(),
                answers: vec![
                    answer(
                        "local",
                        vec![record("A", "192.168.1.60"), record("AAAA", "fd00::60")],
                    ),
                    answer("vpn", vec![record("A", "10.8.0.1")]),
                    answer("office", Vec::new()),
                ],
            },
        ],
        environments: vec!["local".into(), "office".into(), "vpn".into()],
        unaddressed: vec!["office".into()],
        tls_host: Some("portal.home".into()),
        doh_url: Some("https://portal.home/dns-query".into()),
        settings: DnsSettingsResponse {
            address: "0.0.0.0".into(),
            port: 53,
            zones: vec!["home".into()],
            ttl: 60,
            addresses: BTreeMap::from([("vpn".into(), vec!["10.8.0.1".into()])]),
            tls: DnsTlsSettingsResponse {
                enabled: true,
                port: 853,
                certificate: None,
                key: None,
            },
            https: DnsHttpsSettingsResponse {
                enabled: true,
                host: None,
            },
        },
    };
    check("dns", serde_json::to_value(&dns).unwrap());
}
