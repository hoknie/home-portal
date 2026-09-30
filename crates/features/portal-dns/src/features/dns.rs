use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::Method;
use axum::routing::get;
use portal_config::ConfigStore;
use portal_feature::{Action, Area, Feature, Loop, Right, Rule, Validator};

use crate::controllers::{change, resolve_encoded, resolve_posted, show};
use crate::loops::DnsRuntime;
use crate::ports::DnsSources;
use crate::services::{Library, validate_dns};
use crate::types::{Cadence, DnsContext};
use crate::usecases::{ChangeDns, ShowDns};

pub struct DnsFeature {
    context: DnsContext,
    runtime: Arc<DnsRuntime>,
}

impl DnsFeature {
    pub const NAME: &'static str = "dns";
    pub const PATH: &'static str = "/api/dns";
    pub const QUERY_PATH: &'static str = "/dns-query";

    pub fn new(configuration: Arc<ConfigStore>, sources: Arc<dyn DnsSources>) -> DnsFeature {
        Self::with(configuration, sources, Cadence::STANDARD)
    }

    pub fn with(
        configuration: Arc<ConfigStore>,
        sources: Arc<dyn DnsSources>,
        cadence: Cadence,
    ) -> DnsFeature {
        let runtime = Arc::new(DnsRuntime {
            configuration: configuration.clone(),
            sources,
            library: Arc::new(Library::default()),
            cadence,
        });
        runtime.refresh_now();
        DnsFeature {
            context: DnsContext {
                show: ShowDns::new(configuration.clone(), runtime.clone()),
                change: ChangeDns::new(configuration, runtime.clone()),
            },
            runtime,
        }
    }
}

impl Feature for DnsFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::PATH, get(show).put(change))
            .with_state(self.context.clone())
    }

    fn rules(&self) -> Vec<Rule> {
        vec![
            Rule::needs(
                Method::GET,
                Self::PATH,
                &[Right {
                    area: Area::Dns,
                    action: Action::Read,
                }],
            ),
            Rule::needs(
                Method::PUT,
                Self::PATH,
                &[Right {
                    area: Area::Dns,
                    action: Action::Update,
                }],
            ),
        ]
    }

    fn public_router(&self) -> Router {
        Router::new()
            .route(Self::QUERY_PATH, get(resolve_encoded).post(resolve_posted))
            .layer(DefaultBodyLimit::max(Cadence::LARGEST_MESSAGE))
            .with_state(self.runtime.library.clone())
    }

    fn validator(&self) -> Option<Validator> {
        Some(validate_dns)
    }

    fn loops(&self) -> Vec<Loop> {
        vec![Box::pin(self.runtime.clone().run())]
    }
}
