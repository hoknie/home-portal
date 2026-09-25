use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post, put};
use portal_config::{ConfigStore, Storage};
use portal_feature::{Feature, Loop, Validator};
use portal_model::{Environment, ServiceStatus};
use time::OffsetDateTime;

use crate::controllers::{create, history, list, probe_now, remove, update};
use crate::loops::HistoryWriter;
use crate::probes::Probe;
use crate::repositories::HistoryFiles;
use crate::services::{StatusBoard, Supervisor, validate_services};
use crate::types::{ServiceEntry, ServicesPorts, ServicesSection, ServicesState};

pub struct ServicesFeature {
    state: ServicesState,
    history: Arc<HistoryWriter>,
}

impl ServicesFeature {
    pub const NAME: &'static str = "services";
    pub const COLLECTION: &'static str = "/api/services";
    pub const ITEM: &'static str = "/api/services/{id}";
    pub const PROBE: &'static str = "/api/services/{id}/probe";
    pub const HISTORY: &'static str = "/api/services/{id}/history";

    pub fn new(
        configuration: Arc<ConfigStore>,
        host: Environment,
        ports: ServicesPorts,
    ) -> Result<ServicesFeature, String> {
        let board = Arc::new(StatusBoard::watched(
            OffsetDateTime::now_utc(),
            ports.observers,
        ));
        let files = Arc::new(HistoryFiles::at(configuration.storage(Storage::History)));
        let configured: Vec<String> = ServicesSection::read(&configuration.read().document)
            .map(|section| section.services)
            .unwrap_or_default()
            .into_iter()
            .map(|entry| entry.id)
            .collect();
        board.restore(files.load(&configured));
        let history = Arc::new(HistoryWriter::new(board.clone(), files));
        let probe = Arc::new(Probe::new()?);
        let supervisor = Arc::new(Supervisor::new(
            configuration.clone(),
            host,
            board.clone(),
            probe,
        ));
        Ok(ServicesFeature {
            state: ServicesState {
                configuration,
                board,
                supervisor,
                publishing: ports.publishing,
                events: ports.events,
            },
            history,
        })
    }
}

impl ServicesFeature {
    pub fn entries(&self) -> Vec<ServiceEntry> {
        ServicesSection::read(&self.state.configuration.read().document)
            .map(|section| section.services)
            .unwrap_or_default()
    }

    pub fn host(&self) -> &Environment {
        self.state.supervisor.host()
    }

    pub fn publishing(&self) -> Option<u16> {
        self.state.publishing.https_port()
    }

    pub fn status_of(&self, id: &str) -> ServiceStatus {
        self.state.board.status(id)
    }
}

impl Feature for ServicesFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::COLLECTION, get(list).post(create))
            .route(Self::ITEM, put(update).delete(remove))
            .route(Self::PROBE, post(probe_now))
            .route(Self::HISTORY, get(history))
            .with_state(self.state.clone())
    }

    fn validator(&self) -> Option<Validator> {
        Some(validate_services)
    }

    fn loops(&self) -> Vec<Loop> {
        vec![
            Box::pin(self.state.supervisor.clone().run()),
            Box::pin(self.history.clone().run()),
        ]
    }

    fn stop(&self) {
        self.history.flush();
    }
}
