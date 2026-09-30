use std::collections::BTreeMap;
use std::sync::Arc;

use portal_feature::{ApiError, FieldError};

use crate::services::{AutomationSink, WebhookBook, received};
use crate::types::Webhook;

pub const MODULE_OFF: &str = "the webhooks module is off";
pub const STOPPING: &str = "the portal is stopping";
pub const FROM_INTERFACE: &str = "interface";

#[derive(Clone)]
pub struct RunWebhook {
    sink: Arc<AutomationSink>,
    book: Arc<WebhookBook>,
}

impl RunWebhook {
    pub fn new(sink: Arc<AutomationSink>, book: Arc<WebhookBook>) -> RunWebhook {
        RunWebhook { sink, book }
    }

    pub fn run(
        &self,
        id: &str,
        given: &BTreeMap<String, String>,
        by: &str,
    ) -> Result<Option<u64>, ApiError> {
        let cache = &self.sink.cache;
        let webhook = cache
            .webhook(id)
            .filter(|webhook| webhook.enabled)
            .ok_or(ApiError::NotFound(Webhook::UNKNOWN))?;
        if !cache.webhooks_on() {
            return Err(ApiError::Conflict(MODULE_OFF.to_string()));
        }
        if self.sink.stopping() {
            return Err(ApiError::ServiceUnavailable(STOPPING.to_string()));
        }
        let variables = declared(&webhook, given)?;
        let answer = received(
            &self.sink,
            &self.book,
            &webhook,
            &variables,
            (FROM_INTERFACE, Some(by.to_string())),
        );
        let status = match &answer {
            Ok(_) => 202,
            Err(error) => error.status().as_u16(),
        };
        self.book.record(&webhook.id, status);
        answer
    }
}

fn declared(
    webhook: &Webhook,
    given: &BTreeMap<String, String>,
) -> Result<Vec<(String, String)>, ApiError> {
    let mut found = Vec::new();
    let mut missing = Vec::new();
    for name in &webhook.variables {
        match given.get(name) {
            Some(value) => found.push((name.clone(), value.clone())),
            None => missing.push(FieldError::new(name.clone(), "is required")),
        }
    }
    if missing.is_empty() {
        Ok(found)
    } else {
        Err(ApiError::Invalid(missing))
    }
}
