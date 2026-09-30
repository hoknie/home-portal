use std::sync::Arc;

use axum::Router;
use axum::http::Method;
use axum::routing::{get, post, put};
use portal_config::{ConfigStore, Storage};
use portal_feature::{Action, Area, Feature, Loop, Right, Rule, Validator};
use portal_model::{Environment, ServiceStatus};
use time::OffsetDateTime;

use crate::controllers::{create, history, list, probe_now, remove, update};
use crate::loops::HistoryWriter;
use crate::probes::Probe;
use crate::repositories::HistoryFiles;
use crate::services::{Showcase, StatusBoard, Supervisor, validate_services};
use crate::types::{ServiceEntry, ServicesPorts, ServicesSection, ServicesState};
use crate::usecases::{
    ChangeService, CreateService, CurrentStatus, DeleteService, ListServices, ProbeService,
    ServiceEntries, ShowHistory, WakeProbe,
};

pub struct ServicesFeature {
    state: ServicesState,
    showcase: Showcase,
    entries: ServiceEntries,
    history: Arc<HistoryWriter>,
    probe: ProbeService,
    current: CurrentStatus,
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
        let showcase = Showcase {
            board: board.clone(),
            supervisor: supervisor.clone(),
            publishing: ports.publishing,
        };
        let events = ports.events;
        Ok(ServicesFeature {
            state: ServicesState {
                list: ListServices::new(configuration.clone(), showcase.clone()),
                create: CreateService::new(configuration.clone(), showcase.clone(), events.clone()),
                change: ChangeService::new(configuration.clone(), showcase.clone(), events.clone()),
                delete: DeleteService::new(configuration.clone(), showcase.clone(), events),
                wake: WakeProbe::new(configuration.clone(), supervisor),
                history: ShowHistory::new(configuration.clone(), board),
            },
            probe: ProbeService::new(configuration.clone(), showcase.clone()),
            current: CurrentStatus::new(configuration.clone(), showcase.clone()),
            showcase,
            entries: ServiceEntries::new(configuration),
            history,
        })
    }
}

impl ServicesFeature {
    pub fn entries(&self) -> Vec<ServiceEntry> {
        self.entries.run().unwrap_or_default()
    }

    pub fn host(&self) -> &Environment {
        self.showcase.supervisor.host()
    }

    pub fn publishing(&self) -> Option<u16> {
        self.showcase.publishing.https_port()
    }

    pub fn probe_service(&self) -> ProbeService {
        self.probe.clone()
    }

    pub fn current_status(&self) -> CurrentStatus {
        self.current.clone()
    }

    pub fn status_of(&self, id: &str) -> ServiceStatus {
        self.showcase.board.status(id)
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

    fn rules(&self) -> Vec<Rule> {
        vec![
            Rule::signed(Method::GET, Self::COLLECTION),
            Rule::needs(
                Method::POST,
                Self::COLLECTION,
                &[Right {
                    area: Area::Services,
                    action: Action::Create,
                }],
            ),
            Rule::needs(
                Method::PUT,
                Self::ITEM,
                &[Right {
                    area: Area::Services,
                    action: Action::Update,
                }],
            ),
            Rule::needs(
                Method::DELETE,
                Self::ITEM,
                &[Right {
                    area: Area::Services,
                    action: Action::Delete,
                }],
            ),
            Rule::needs(
                Method::POST,
                Self::PROBE,
                &[Right {
                    area: Area::Services,
                    action: Action::Update,
                }],
            ),
            Rule::signed(Method::GET, Self::HISTORY),
        ]
    }

    fn validator(&self) -> Option<Validator> {
        Some(validate_services)
    }

    fn loops(&self) -> Vec<Loop> {
        vec![
            Box::pin(self.showcase.supervisor.clone().run()),
            Box::pin(self.history.clone().run()),
        ]
    }

    fn stop(&self) {
        self.history.flush();
    }
}
