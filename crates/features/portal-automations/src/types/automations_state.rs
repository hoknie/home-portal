use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::ports::Directory;
use crate::services::{AutomationSink, ScriptsDirectory, WebhookBook};
use crate::usecases::{
    ChangeAutomation, ChangeWebhook, CreateAutomation, CreateWebhook, DeleteAutomation,
    DeleteWebhook, IssueToken, ListAutomations, ListWebhooks, RemoveToken,
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
    pub scripts: ScriptsDirectory,
    pub manual_runs: Arc<Mutex<HashMap<String, Instant>>>,
    pub webhooks: Arc<WebhookBook>,
}
