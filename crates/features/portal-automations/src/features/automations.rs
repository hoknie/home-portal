use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post, put};
use portal_config::{ConfigStore, Storage};
use portal_feature::{EventSink, Feature, Loop, StatusObserver, Validator};

use crate::controllers::{
    catalogue, create, create_webhook, delete_webhook, issue_token, list, list_webhooks, receive,
    remove, remove_token, run, run_now, runs, schedule, scripts, stop, update, update_webhook,
};
use crate::loops::{JournalWriter, dispatch_forever, schedule_forever, watch_forever};
use crate::ports::{Clock, Directory};
use crate::repositories::RunFile;
use crate::services::{
    AutomationCache, AutomationSink, Journal, ScriptsDirectory, StatusRelay, SystemClock,
    validate_automations,
};
use crate::types::AutomationsState;

pub struct AutomationsFeature {
    pub(crate) state: AutomationsState,
    clock: Arc<dyn Clock>,
    writer: Arc<JournalWriter>,
}

impl AutomationsFeature {
    pub const NAME: &'static str = "automations";
    pub const COLLECTION: &'static str = "/api/automations";
    pub const ITEM: &'static str = "/api/automations/{id}";
    pub const RUN: &'static str = "/api/automations/{id}/run";
    pub const RUNS: &'static str = "/api/automations/runs";
    pub const RUN_ITEM: &'static str = "/api/automations/runs/{id}";
    pub const RUN_STOP: &'static str = "/api/automations/runs/{id}/stop";
    pub const CATALOGUE: &'static str = "/api/automations/catalogue";
    pub const SCRIPTS: &'static str = "/api/automations/scripts";
    pub const SCHEDULE: &'static str = "/api/automations/schedule";
    pub const WEBHOOKS: &'static str = "/api/webhooks";
    pub const WEBHOOK: &'static str = "/api/webhooks/{id}";
    pub const WEBHOOK_TOKEN: &'static str = "/api/webhooks/{id}/token";
    pub const RECEIVE: &'static str = "/webhook/{id}";
    pub const LARGEST_BODY: usize = 64 * 1024;

    pub fn new(
        configuration: Arc<ConfigStore>,
        directory: Arc<dyn Directory>,
    ) -> AutomationsFeature {
        let cache = Arc::new(AutomationCache::of(&configuration.read().document));
        let scripts = ScriptsDirectory::at(configuration.storage(Storage::Scripts));
        let file = Arc::new(RunFile::at(&configuration.storage(Storage::Automations)));
        let sink = Arc::new(AutomationSink::restored(cache, file.load(Journal::KEPT)));
        let writer = Arc::new(JournalWriter::new(sink.journal.clone(), file));
        AutomationsFeature {
            state: AutomationsState {
                configuration,
                sink,
                directory,
                scripts,
                manual_runs: Arc::new(Mutex::new(HashMap::new())),
                webhooks: Arc::new(crate::services::WebhookBook::default()),
            },
            clock: Arc::new(SystemClock),
            writer,
        }
    }

    pub fn events(&self) -> Arc<dyn EventSink> {
        self.state.sink.clone()
    }

    pub fn observer(&self) -> Arc<dyn StatusObserver> {
        Arc::new(StatusRelay {
            sink: self.state.sink.clone(),
        })
    }
}

impl Feature for AutomationsFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::COLLECTION, get(list).post(create))
            .route(Self::ITEM, put(update).delete(remove))
            .route(Self::RUN, post(run_now))
            .route(Self::RUNS, get(runs))
            .route(Self::RUN_ITEM, get(run))
            .route(Self::RUN_STOP, post(stop))
            .route(Self::CATALOGUE, get(catalogue))
            .route(Self::SCRIPTS, get(scripts))
            .route(Self::SCHEDULE, get(schedule))
            .route(Self::WEBHOOKS, get(list_webhooks).post(create_webhook))
            .route(Self::WEBHOOK, put(update_webhook).delete(delete_webhook))
            .route(Self::WEBHOOK_TOKEN, post(issue_token).delete(remove_token))
            .with_state(self.state.clone())
    }

    fn public_router(&self) -> Router {
        Router::new()
            .route(Self::RECEIVE, post(receive))
            .layer(DefaultBodyLimit::max(Self::LARGEST_BODY))
            .with_state(self.state.clone())
    }

    fn validator(&self) -> Option<Validator> {
        Some(validate_automations)
    }

    fn loops(&self) -> Vec<Loop> {
        vec![
            Box::pin(dispatch_forever(
                self.state.sink.clone(),
                self.state.scripts.clone(),
            )),
            Box::pin(schedule_forever(
                self.state.sink.clone(),
                self.clock.clone(),
            )),
            Box::pin(self.writer.clone().run()),
            Box::pin(watch_forever(
                self.state.configuration.clone(),
                self.state.sink.clone(),
            )),
        ]
    }

    fn stop(&self) {
        self.writer.flush();
    }
}
