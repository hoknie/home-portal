use super::*;
use crate::typed;

#[test]
fn the_session_sample_matches_its_serializer() {
    typed(
        "session",
        &SessionResponse::of(&Principal::member(
            "anna",
            Some("family".to_string()),
            Rights::of([
                Right::new(Area::Automations, Action::Execute),
                Right::new(Area::Automations, Action::Read),
            ]),
        )),
    );
}

#[test]
fn the_services_sample_matches_its_serializer() {
    let start = datetime!(2026-09-22 10:00 UTC);
    let up = ServiceStatus::unknown(start).after(
        ProbeOutcome::answered(ServiceState::Up, 42),
        datetime!(2026-09-22 10:00:30 UTC),
    );
    let down = up.after(
        ProbeOutcome::failed(ServiceState::Down, None, "connection refused".into())
            .because(Diagnosis::Refused),
        datetime!(2026-09-22 10:01 UTC),
    );
    let mut media = entry("media", "Media", "http://192.168.1.10:8096");
    media.group = Some("Media".into());
    media.icon = Some("film".into());
    media.description = Some("Films and series".into());
    media.links = vec![portal_services::ServiceLink {
        title: "Admin".into(),
        url: "http://192.168.1.10:8096/web/#/dashboard".into(),
    }];
    media.notes = Some("Films live on the NAS.\n\n**Restart:** `docker restart jellyfin`".into());
    media.widgets = vec!["box".into()];
    media
        .addresses
        .insert("local".into(), "http://192.168.1.10:8096".into());
    media
        .addresses
        .insert("internet".into(), "https://media.example.com".into());
    let local = Environment::parse("local").unwrap();
    let mut nas = entry("nas", "NAS", "https://nas.local");
    nas.probe.every_seconds = 60;
    let mut publication = portal_model::Publication::new("nas.example.com");
    publication.auth = vec!["internet".into()];
    nas.proxy = Some(publication);
    let mut printer = entry("printer", "Printer", "tcp://192.168.1.30");
    printer.probe.enabled = false;
    printer.probe.kind = ProbeKind::Tcp;
    printer.probe.port = Some(9100);
    let viewpoint = portal_services::Viewpoint {
        environment: &local,
        host: &local,
        publishing: Some(443),
    };
    let response = ServicesResponse {
        services: vec![
            ServiceResponse::of(media, viewpoint, up),
            ServiceResponse::of(nas, viewpoint, down),
            ServiceResponse::of(printer, viewpoint, ServiceStatus::unknown(start)),
        ],
    };
    typed("services", &response);
}

#[test]
fn the_network_sample_matches_its_serializer() {
    let configured = NetworkSettings {
        address: "0.0.0.0".parse().unwrap(),
        port: 9090,
        public_url: Some("https://portal.home.lan".into()),
        trusted_proxies: vec!["10.0.0.0/8".parse().unwrap()],
    };
    let effective = EffectiveAddress {
        address: "127.0.0.1:8080".parse().unwrap(),
        overridden: false,
    };
    let interfaces = vec![
        InterfaceResponse {
            name: "en0".into(),
            addresses: vec!["192.168.1.20".into(), "fe80::1".into()],
        },
        InterfaceResponse {
            name: "lo0".into(),
            addresses: vec!["127.0.0.1".into()],
        },
    ];
    typed(
        "network",
        &NetworkResponse::of(configured, effective, interfaces),
    );
}

#[test]
fn the_dashboard_sample_matches_its_serializer() {
    let placed = |kind: &str, section: &str, width: i64| WidgetInstance {
        section: Some(section.into()),
        width: Some(width),
        ..WidgetInstance::of(kind)
    };
    let widgets = vec![
        WidgetInstance {
            appearance: WidgetAppearance {
                surface: Some(Surface::Plain),
                title: Some(TitleVisibility::Hidden),
                ..WidgetAppearance::default()
            },
            id: Some("status-summary".into()),
            ..placed("status-summary", "now", 8)
        },
        WidgetInstance {
            id: Some("riga".into()),
            settings: json!({ "city": "Riga" }),
            public: true,
            height: WidgetHeight::Rows(3),
            appearance: WidgetAppearance {
                surface: Some(Surface::Tinted),
                accent: Some(Accent::Blue),
                ..WidgetAppearance::default()
            },
            column: Some(9),
            row: Some(1),
            ..placed("weather", "now", 4)
        },
        WidgetInstance {
            title: Some("Media".into()),
            settings: json!({ "groups": ["Media"] }),
            id: Some("services".into()),
            environments: Some(vec!["local".into()]),
            ..placed("services", "media", 6)
        },
        WidgetInstance {
            id: Some("services-2".into()),
            ..placed("services", "media", 5)
        },
    ];
    typed(
        "dashboard",
        &DashboardResponse {
            sections: vec![
                SectionView {
                    id: "now".into(),
                    title: Some("Now".into()),
                    appearance: ResolvedSectionAppearance::default(),
                },
                SectionView {
                    id: "media".into(),
                    title: Some("Media".into()),
                    appearance: ResolvedSectionAppearance {
                        title: TitleVisibility::Hidden,
                        surface: SectionSurface::SectionCard,
                    },
                },
            ],
            widgets: widgets
                .into_iter()
                .enumerate()
                .map(|(index, widget)| WidgetView::of(WidgetView::key_of(index), widget))
                .collect(),
        },
    );
}

#[test]
fn the_widget_library_sample_matches_its_serializer() {
    let weather = WidgetInstance {
        id: Some("riga".into()),
        title: Some("Riga".into()),
        settings: json!({ "latitude": 56.95, "longitude": 24.11 }),
        public: true,
        appearance: WidgetAppearance {
            surface: Some(Surface::Tinted),
            accent: Some(Accent::Blue),
            ..WidgetAppearance::default()
        },
        ..WidgetInstance::of("weather")
    };
    let custom = WidgetInstance {
        id: Some("disks".into()),
        settings: json!({ "source": { "workflow": "disks" }, "blocks": [{ "kind": "stat", "label": "Free", "value": "{{data.free}}" }] }),
        environments: Some(vec!["local".into()]),
        ..WidgetInstance::of("custom")
    };
    typed(
        "dashboard-library",
        &LibraryResponse {
            widgets: vec![
                LibraryWidgetView::of(weather, 1),
                LibraryWidgetView::of(custom, 0),
            ],
        },
    );
}

#[tokio::test]
async fn the_field_errors_sample_matches_its_serializer() {
    let response = ApiError::Invalid(vec![
        FieldError::new("id", "must start with a lower-case letter"),
        FieldError::new("url", "must use http or https"),
    ])
    .into_response();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    check("field-errors", serde_json::from_slice(&bytes).unwrap());
}

#[test]
fn the_environment_sample_matches_its_serializer() {
    typed(
        "environment",
        &EnvironmentResponse {
            environment: Environment::parse("local").unwrap(),
            detected: Environment::parse("local").unwrap(),
            switchable: true,
            environments: vec![
                Environment::parse("local").unwrap(),
                Environment::parse("vpn").unwrap(),
                Environment::internet(),
            ],
        },
    );
}

#[test]
fn the_icons_sample_matches_its_serializer() {
    typed(
        "icons",
        &vec![
            IconState {
                service: "media".into(),
                source: "auto".into(),
                available: true,
                fetched_at: Some(datetime!(2026-09-22 10:00 UTC)),
                problem: None,
            },
            IconState {
                service: "router".into(),
                source: "catalog:tplink".into(),
                available: false,
                fetched_at: None,
                problem: Some("the catalogue answered 404".into()),
            },
        ],
    );
}

#[test]
fn the_secrets_sample_matches_its_serializer() {
    typed(
        "secrets",
        &SecretsResponse {
            secrets: vec![
                SecretResponse {
                    name: "telegram_token".into(),
                    set: true,
                },
                SecretResponse {
                    name: "calendar_password".into(),
                    set: false,
                },
            ],
        },
    );
}
