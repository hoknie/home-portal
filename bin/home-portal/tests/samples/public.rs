use super::*;

#[test]
fn the_public_portal_sample_matches_its_serializer() {
    let status = ServiceStatus::unknown(datetime!(2026-09-22 10:00 UTC)).after(
        ProbeOutcome::answered(ServiceState::Up, 42),
        datetime!(2026-09-22 10:00:30 UTC),
    );
    check(
        "public-portal",
        serde_json::to_value(PortalResponse {
            environment: Environment::parse("local").unwrap(),
            detected: Environment::parse("local").unwrap(),
            switchable: true,
            environments: Some(vec![
                Environment::parse("local").unwrap(),
                Environment::parse("vpn").unwrap(),
                Environment::internet(),
            ]),
            sections: vec![PublicSection {
                id: "now".into(),
                title: Some("Now".into()),
            }],
            services: vec![PublicService {
                id: "media".into(),
                name: "Media".into(),
                address: "http://192.168.1.10:8096".into(),
                group: Some("Media".into()),
                icon: Some("/api/public/icons/media".into()),
                description: Some("Films and series".into()),
                status: Some(status),
            }],
            widgets: vec![PublicWidget {
                kind: "weather".into(),
                id: Some("riga".into()),
                title: None,
                settings: json!({ "city": "Riga" }),
                section: Some("now".into()),
                size: WidgetSize::Third,
            }],
        })
        .unwrap(),
    );
}
