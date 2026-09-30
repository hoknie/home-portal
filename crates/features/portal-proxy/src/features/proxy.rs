use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::http::Method;
use axum::routing::{get, post, put};
use portal_config::{ConfigStore, Storage};
use portal_feature::{Action, Area, Feature, FieldError, Loop, Right, Rule, Validator};
use toml_edit::DocumentMut;

use crate::controllers::{
    apply, authorize, change, change_source, download, resume, root_certificate, show, start, stop,
};
use crate::loops::CaddySync;
use crate::renderers::AUTHORIZE_PATH;
use crate::services::{CaddyManager, validate_publications, validate_settings};
use crate::types::{CaddyHome, Cadence, ProxyPorts, ProxyState, SyncSetup};
use crate::usecases::{
    ApplyProxy, ChangeCaddySource, ChangeProxy, DownloadCaddy, ShowProxy, StartCaddy, StopCaddy,
};

pub struct ProxyFeature {
    state: ProxyState,
}

impl ProxyFeature {
    pub const NAME: &'static str = "proxy";
    pub const PATH: &'static str = "/api/proxy";
    pub const APPLY_PATH: &'static str = "/api/proxy/apply";
    pub const AUTHORIZE_PATH: &'static str = AUTHORIZE_PATH;
    pub const CONTINUE_PATH: &'static str = "/api/proxy/continue";
    pub const ROOT_CERTIFICATE_PATH: &'static str = "/api/proxy/root-certificate";
    pub const DOWNLOAD_PATH: &'static str = "/api/proxy/caddy/download";
    pub const START_PATH: &'static str = "/api/proxy/caddy/start";
    pub const STOP_PATH: &'static str = "/api/proxy/caddy/stop";
    pub const SOURCE_PATH: &'static str = "/api/proxy/caddy";

    pub fn new(
        configuration: Arc<ConfigStore>,
        ports: ProxyPorts,
        portal: SocketAddr,
    ) -> ProxyFeature {
        let manager = Arc::new(CaddyManager::new(CaddyHome::at(
            configuration.storage(Storage::Caddy),
        )));
        let setup = SyncSetup {
            portal,
            cadence: Cadence::STANDARD,
            manager,
        };
        Self::with(configuration, ports, setup)
    }

    pub fn with(
        configuration: Arc<ConfigStore>,
        ports: ProxyPorts,
        setup: SyncSetup,
    ) -> ProxyFeature {
        let sync = Arc::new(CaddySync::new(
            configuration.clone(),
            ports.services.clone(),
            setup,
        ));
        ProxyFeature {
            state: ProxyState {
                show: ShowProxy::new(configuration.clone(), sync.clone()),
                apply: ApplyProxy::new(configuration.clone(), sync.clone()),
                change: ChangeProxy::new(configuration.clone(), sync.clone()),
                download: DownloadCaddy::new(configuration.clone(), sync.clone()),
                start: StartCaddy::new(configuration.clone(), sync.clone()),
                stop: StopCaddy::new(configuration.clone(), sync.clone()),
                change_source: ChangeCaddySource::new(configuration, sync.clone()),
                ports,
                sync,
            },
        }
    }

    pub fn validate(document: &DocumentMut) -> Vec<FieldError> {
        let mut errors = validate_settings(document);
        errors.extend(validate_publications(document));
        errors
    }
}

impl Feature for ProxyFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::PATH, get(show).put(change))
            .route(Self::APPLY_PATH, post(apply))
            .route(Self::DOWNLOAD_PATH, post(download))
            .route(Self::START_PATH, post(start))
            .route(Self::STOP_PATH, post(stop))
            .route(Self::SOURCE_PATH, put(change_source))
            .with_state(self.state.clone())
    }

    fn rules(&self) -> Vec<Rule> {
        vec![
            Rule::needs(
                Method::GET,
                Self::PATH,
                &[Right {
                    area: Area::Proxy,
                    action: Action::Read,
                }],
            ),
            Rule::needs(
                Method::PUT,
                Self::PATH,
                &[Right {
                    area: Area::Proxy,
                    action: Action::Update,
                }],
            ),
            Rule::needs(
                Method::POST,
                Self::APPLY_PATH,
                &[Right {
                    area: Area::Proxy,
                    action: Action::Update,
                }],
            ),
            Rule::needs(
                Method::POST,
                Self::DOWNLOAD_PATH,
                &[Right {
                    area: Area::Proxy,
                    action: Action::Update,
                }],
            ),
            Rule::needs(
                Method::POST,
                Self::START_PATH,
                &[Right {
                    area: Area::Proxy,
                    action: Action::Update,
                }],
            ),
            Rule::needs(
                Method::POST,
                Self::STOP_PATH,
                &[Right {
                    area: Area::Proxy,
                    action: Action::Update,
                }],
            ),
            Rule::admin(Method::PUT, Self::SOURCE_PATH),
        ]
    }

    fn public_router(&self) -> Router {
        Router::new()
            .route(Self::AUTHORIZE_PATH, get(authorize))
            .route(Self::CONTINUE_PATH, get(resume))
            .route(Self::ROOT_CERTIFICATE_PATH, get(root_certificate))
            .with_state(self.state.clone())
    }

    fn validator(&self) -> Option<Validator> {
        Some(Self::validate)
    }

    fn loops(&self) -> Vec<Loop> {
        vec![Box::pin(self.state.sync.clone().run())]
    }
}
