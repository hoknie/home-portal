use portal_model::TlsMode;
use portal_model::TlsPolicy;
use portal_proxy::{
    CaddyResponse, DownloadStage, DownloadState, ProxyResponse, RouteResponse, SettingsResponse,
};
use time::macros::datetime;

use crate::check;

#[test]
fn the_proxy_sample_matches_its_serializer() {
    let route = |host: &str, service: Option<&str>, upstream: &str, tls: TlsMode| RouteResponse {
        host: host.into(),
        address: format!("https://{host}"),
        service: service.map(str::to_string),
        upstream: upstream.into(),
        tls,
        auth: Vec::new(),
        environments: Vec::new(),
    };
    let mut nas = route(
        "nas.example.com",
        Some("nas"),
        "http://192.168.1.5",
        TlsMode::Acme,
    );
    nas.auth = vec!["internet".into()];
    nas.environments = vec!["internet".into()];
    let mut media = route(
        "media.home.arpa",
        Some("media"),
        "http://192.168.1.10:8096",
        TlsMode::Internal,
    );
    media.environments = vec!["local".into()];
    let response = ProxyResponse {
        enabled: true,
        settings: SettingsResponse {
            http_port: 80,
            https_port: 443,
            portal_host: Some("portal.example.com".into()),
            cookie_domain: Some("example.com".into()),
            tls: TlsPolicy {
                email: Some("owner@example.com".into()),
                ..TlsPolicy::default()
            },
        },
        admin: "http://127.0.0.1:2019".into(),
        reachable: true,
        in_sync: true,
        last_applied_at: Some(datetime!(2026-09-24 10:00 UTC)),
        last_error: None,
        routes: vec![
            route(
                "portal.example.com",
                None,
                "http://127.0.0.1:8080",
                TlsMode::Acme,
            ),
            nas,
            media,
        ],
        caddy: CaddyResponse {
            managed: true,
            installed: Some("2.11.4".into()),
            installed_from: Some(
                "https://github.com/caddyserver/caddy/releases/download/v2.11.4/caddy_2.11.4_mac_arm64.tar.gz"
                    .into(),
            ),
            source: "https://api.github.com/repos/caddyserver/caddy/releases".into(),
            version: "latest".into(),
            release_url: "https://api.github.com/repos/caddyserver/caddy/releases/latest".into(),
            platform: Some("mac_arm64".into()),
            platform_error: None,
            download: DownloadState {
                state: DownloadStage::Idle,
                error: None,
            },
            log: vec!["{\"level\":\"info\",\"msg\":\"serving initial configuration\"}".into()],
        },
    };
    check("proxy", serde_json::to_value(response).unwrap());
}
