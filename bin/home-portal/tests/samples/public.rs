use super::*;
use crate::typed;

#[test]
fn the_public_portal_sample_matches_its_serializer() {
    let status = ServiceStatus::unknown(datetime!(2026-09-22 10:00 UTC)).after(
        ProbeOutcome::answered(ServiceState::Up, 42),
        datetime!(2026-09-22 10:00:30 UTC),
    );
    typed(
        "public-portal",
        &PortalResponse {
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
                appearance: ResolvedSectionAppearance::default(),
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
                width: 4,
                column: None,
                row: None,
                height: WidgetHeight::Auto,
                appearance: ResolvedAppearance::default(),
            }],
        },
    );
}

#[test]
fn the_failure_report_sample_matches_its_serializer() {
    typed(
        "failure-report",
        &home_portal::FailureReportResponse {
            since: datetime!(2026-10-02 09:00 UTC),
            checked: datetime!(2026-10-02 09:05:12 UTC),
            details: true,
            problems: Some(vec![
                home_portal::ProblemResponse {
                    file: Some("/Users/ad/.config/home-portal/home-portal.toml".into()),
                    field: Some("proxy.enabled".into()),
                    message: "is no longer read; switch the module with modules.proxy".into(),
                },
                home_portal::ProblemResponse {
                    file: Some("/Users/ad/.config/home-portal/services.toml".into()),
                    field: None,
                    message: "configuration file /Users/ad/.config/home-portal/services.toml is not valid TOML: expected `]` at line 3".into(),
                },
            ]),
        },
    );
}
