use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use portal_config::ConfigStore;

use crate::ports::Directory;
use crate::services::{AutomationSink, ScriptsDirectory, WebhookBook};

#[derive(Clone)]
pub struct AutomationsState {
    pub configuration: Arc<ConfigStore>,
    pub sink: Arc<AutomationSink>,
    pub directory: Arc<dyn Directory>,
    pub scripts: ScriptsDirectory,
    pub manual_runs: Arc<Mutex<HashMap<String, Instant>>>,
    pub webhooks: Arc<WebhookBook>,
}
