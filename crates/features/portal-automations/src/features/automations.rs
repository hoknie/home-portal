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
use crate::controllers::{
    create_workflow, delete_workflow, list_workflows, run_workflow, update_workflow,
    workflow_catalogue,
};
use crate::loops::{
    JournalWriter, dispatch_children_forever, dispatch_forever, schedule_forever, watch_forever,
};
use crate::ports::{Clock, Directory, PortalActions};
use crate::repositories::RunFile;
use crate::services::{
    AutomationCache, AutomationSink, Journal, ScriptsDirectory, StatusRelay, SystemClock, Views,
    WebhookBook, WebhookWriter, WorkflowTools, validate_automations,
};
use crate::types::{AutomationsState, WorkflowCases};
use crate::usecases::{
    ChangeAutomation, ChangeWebhook, CreateAutomation, CreateWebhook, DeleteAutomation,
    DeleteWebhook, IssueToken, ListAutomations, ListWebhooks, RemoveToken,
};
use crate::usecases::{
    ChangeWorkflow, CreateWorkflow, DeleteWorkflow, ListWorkflows, RunWorkflow, WorkflowCatalogue,
};

pub struct AutomationsFeature {
    pub(crate) state: AutomationsState,
    pub(crate) configuration: Arc<ConfigStore>,
    clock: Arc<dyn Clock>,
    writer: Arc<JournalWriter>,
    tools: WorkflowTools,
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
    pub const WORKFLOWS: &'static str = "/api/workflows";
    pub const WORKFLOW: &'static str = "/api/workflows/{id}";
    pub const WORKFLOW_RUN: &'static str = "/api/workflows/{id}/run";
    pub const WORKFLOW_CATALOGUE: &'static str = "/api/workflows/catalogue";
    pub const LARGEST_BODY: usize = 64 * 1024;

    pub fn new(
        configuration: Arc<ConfigStore>,
        directory: Arc<dyn Directory>,
        actions: Arc<dyn PortalActions>,
    ) -> Result<AutomationsFeature, String> {
        let tools = WorkflowTools::of(configuration.clone(), actions)?;
        let cache = Arc::new(AutomationCache::of(&configuration.read().document));
        let scripts = ScriptsDirectory::at(configuration.storage(Storage::Scripts));
        let file = Arc::new(RunFile::at(&configuration.storage(Storage::Automations)));
        let sink = Arc::new(AutomationSink::restored(cache, file.load(Journal::KEPT)));
        let writer = Arc::new(JournalWriter::new(sink.journal.clone(), file));
        let webhooks = Arc::new(WebhookBook::default());
        let views = Views {
            sink: sink.clone(),
            webhooks: webhooks.clone(),
        };
        let manual_runs = Arc::new(Mutex::new(HashMap::new()));
        let workflows = WorkflowCases {
            list: ListWorkflows::new(configuration.clone(), views.clone()),
            create: CreateWorkflow::new(configuration.clone(), views.clone()),
            change: ChangeWorkflow::new(configuration.clone(), views.clone()),
            delete: DeleteWorkflow::new(configuration.clone(), views.clone()),
            run: RunWorkflow::new(sink.clone(), manual_runs.clone()),
            catalogue: WorkflowCatalogue,
        };
        let webhook_writer = WebhookWriter {
            configuration: configuration.clone(),
            sink: sink.clone(),
        };
        Ok(AutomationsFeature {
            state: AutomationsState {
                list: ListAutomations::new(configuration.clone(), views.clone()),
                create: CreateAutomation::new(configuration.clone(), sink.clone()),
                change: ChangeAutomation::new(configuration.clone(), views.clone()),
                delete: DeleteAutomation::new(configuration.clone(), views.clone()),
                list_webhooks: ListWebhooks::new(configuration.clone(), views.clone()),
                create_webhook: CreateWebhook::new(configuration.clone(), sink.clone()),
                change_webhook: ChangeWebhook::new(
                    configuration.clone(),
                    webhook_writer.clone(),
                    views.clone(),
                ),
                delete_webhook: DeleteWebhook::new(webhook_writer.clone(), views.clone()),
                issue_token: IssueToken::new(webhook_writer.clone()),
                remove_token: RemoveToken::new(webhook_writer, views),
                sink,
                directory,
                scripts,
                manual_runs,
                webhooks,
                workflows,
            },
            configuration,
            clock: Arc::new(SystemClock),
            writer,
            tools,
        })
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
            .route(Self::WORKFLOWS, get(list_workflows).post(create_workflow))
            .route(Self::WORKFLOW, put(update_workflow).delete(delete_workflow))
            .route(Self::WORKFLOW_RUN, post(run_workflow))
            .route(Self::WORKFLOW_CATALOGUE, get(workflow_catalogue))
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
                self.tools.clone(),
            )),
            Box::pin(dispatch_children_forever(
                self.state.sink.clone(),
                self.state.scripts.clone(),
                self.tools.clone(),
            )),
            Box::pin(schedule_forever(
                self.state.sink.clone(),
                self.clock.clone(),
            )),
            Box::pin(self.writer.clone().run()),
            Box::pin(watch_forever(
                self.configuration.clone(),
                self.state.sink.clone(),
            )),
        ]
    }

    fn stop(&self) {
        self.writer.flush();
    }
}
