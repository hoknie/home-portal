use portal_model::TlsMode;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::{CaddyResponse, RouteResponse, SettingsResponse};
use crate::types::ProxyView;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProxyResponse {
    pub enabled: bool,
    pub settings: SettingsResponse,
    pub admin: String,
    pub reachable: bool,
    pub in_sync: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_applied_at: Option<OffsetDateTime>,
    pub last_error: Option<String>,
    pub routes: Vec<RouteResponse>,
    pub caddy: CaddyResponse,
}

impl ProxyResponse {
    pub const PORTAL_SCHEME: &'static str = "http://";

    pub fn of(view: ProxyView) -> ProxyResponse {
        let enabled = view.settings.enabled;
        let routes = if enabled {
            Self::routes(&view)
        } else {
            Vec::new()
        };
        ProxyResponse {
            enabled,
            settings: SettingsResponse {
                http_port: view.settings.http_port,
                https_port: view.settings.https_port,
                portal_host: view.settings.portal_host.clone(),
                cookie_domain: view.settings.cookie_domain.clone(),
                tls: view.settings.tls.clone(),
            },
            admin: view.settings.admin.to_string(),
            reachable: view.state.reachable,
            in_sync: enabled && view.rendered.is_some() && view.state.applied == view.rendered,
            last_applied_at: view.state.last_applied_at,
            last_error: view.state.last_error,
            routes,
            caddy: view.caddy,
        }
    }

    pub fn uses_internal(&self) -> bool {
        self.routes
            .iter()
            .any(|route| route.tls == TlsMode::Internal)
    }

    fn routes(view: &ProxyView) -> Vec<RouteResponse> {
        let settings = &view.settings;
        let portal = settings.portal_host.iter().map(|host| RouteResponse {
            host: host.clone(),
            address: settings.origin(host),
            service: None,
            upstream: format!("{}{}", Self::PORTAL_SCHEME, view.portal),
            tls: settings.tls.mode,
            auth: Vec::new(),
            environments: Vec::new(),
        });
        let published = view.services.iter().map(|service| RouteResponse {
            host: service.publication.host.clone(),
            address: settings.origin(&service.publication.host),
            service: Some(service.id.clone()),
            upstream: service.upstream.clone(),
            tls: settings.tls_of(&service.publication).mode,
            auth: service.publication.auth.clone(),
            environments: service.publication.environments.clone(),
        });
        portal.chain(published).collect()
    }
}
