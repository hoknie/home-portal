use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::ports::{Directory, ScriptLibrary};
use crate::services::{AutomationSink, WebhookBook};
use crate::usecases::{
    ChangeAutomation, ChangeWebhook, ChangeWorkflow, CreateAutomation, CreateWebhook,
    CreateWorkflow, DeleteAutomation, DeleteWebhook, DeleteWorkflow, IssueToken, ListAutomations,
    ListWebhooks, ListWorkflows, ReadPortalValues, RemoveToken, RunWorkflow, WorkflowCatalogue,
};

#[derive(Clone)]
pub struct AutomationsState {
    pub list: ListAutomations,
    pub create: CreateAutomation,
    pub change: ChangeAutomation,
    pub delete: DeleteAutomation,
    pub list_webhooks: ListWebhooks,
    pub create_webhook: CreateWebhook,
    pub change_webhook: ChangeWebhook,
    pub delete_webhook: DeleteWebhook,
    pub issue_token: IssueToken,
    pub remove_token: RemoveToken,
    pub sink: Arc<AutomationSink>,
    pub directory: Arc<dyn Directory>,
    pub scripts: Arc<dyn ScriptLibrary>,
    pub manual_runs: Arc<Mutex<HashMap<String, Instant>>>,
    pub webhooks: Arc<WebhookBook>,
    pub workflows: WorkflowCases,
}

#[derive(Clone)]
pub struct WorkflowCases {
    pub list: ListWorkflows,
    pub create: CreateWorkflow,
    pub change: ChangeWorkflow,
    pub delete: DeleteWorkflow,
    pub run: RunWorkflow,
    pub catalogue: WorkflowCatalogue,
    pub portal: ReadPortalValues,
}
