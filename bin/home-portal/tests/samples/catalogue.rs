use super::*;

#[test]
fn the_session_sample_matches_its_serializer() {
    check(
        "session",
        serde_json::to_value(SessionResponse {
            name: "admin".into(),
        })
        .unwrap(),
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
    check("services", serde_json::to_value(response).unwrap());
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
    check(
        "network",
        serde_json::to_value(NetworkResponse::of(configured, effective, interfaces)).unwrap(),
    );
}

#[test]
fn the_dashboard_sample_matches_its_serializer() {
    let placed = |kind: &str, section: &str, size: WidgetSize| WidgetInstance {
        section: Some(section.into()),
        size,
        ..WidgetInstance::of(kind)
    };
    let widgets = vec![
        placed("status-summary", "now", WidgetSize::TwoThirds),
        WidgetInstance {
            id: Some("riga".into()),
            settings: json!({ "city": "Riga" }),
            public: true,
            ..placed("weather", "now", WidgetSize::Third)
        },
        WidgetInstance {
            title: Some("Media".into()),
            settings: json!({ "groups": ["Media"] }),
            environments: Some(vec!["local".into()]),
            ..placed("services", "media", WidgetSize::Half)
        },
        placed("services", "media", WidgetSize::Half),
    ];
    check(
        "dashboard",
        serde_json::to_value(DashboardResponse {
            sections: vec![
                SectionView {
                    id: "now".into(),
                    title: Some("Сейчас".into()),
                },
                SectionView {
                    id: "media".into(),
                    title: Some("Медиа".into()),
                },
            ],
            widgets: widgets
                .into_iter()
                .enumerate()
                .map(|(index, widget)| WidgetView::of(WidgetView::key_of(index, &widget), widget))
                .collect(),
        })
        .unwrap(),
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
    check(
        "environment",
        serde_json::to_value(EnvironmentResponse {
            environment: Environment::parse("local").unwrap(),
            detected: Environment::parse("local").unwrap(),
            switchable: true,
            environments: vec![
                Environment::parse("local").unwrap(),
                Environment::parse("vpn").unwrap(),
                Environment::internet(),
            ],
        })
        .unwrap(),
    );
}

#[test]
fn the_icons_sample_matches_its_serializer() {
    check(
        "icons",
        serde_json::to_value(vec![
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
        ])
        .unwrap(),
    );
}

#[test]
fn the_secrets_sample_matches_its_serializer() {
    check(
        "secrets",
        serde_json::to_value(SecretsResponse {
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
        })
        .unwrap(),
    );
}
