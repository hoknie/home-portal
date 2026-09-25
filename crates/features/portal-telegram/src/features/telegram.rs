use std::sync::Arc;

use axum::Router;
use portal_config::ConfigStore;
use portal_feature::{Feature, Loop, StatusObserver};

use crate::clients::Bot;
use crate::loops::send_forever;
use crate::services::{Outbox, TelegramNotifier, check_telegram};

pub struct TelegramFeature {
    configuration: Arc<ConfigStore>,
    outbox: Arc<Outbox>,
    bot: Arc<Bot>,
}

impl TelegramFeature {
    pub const NAME: &'static str = "telegram";

    pub fn new(configuration: Arc<ConfigStore>, endpoint: &str) -> Result<TelegramFeature, String> {
        let problems = check_telegram(&configuration.read().document, &configuration);
        if let Some(problem) = problems.first() {
            return Err(format!("{}: {}", problem.field, problem.message));
        }
        Ok(TelegramFeature {
            configuration,
            outbox: Arc::new(Outbox::default()),
            bot: Arc::new(Bot::new(endpoint)?),
        })
    }

    pub fn observer(&self) -> Arc<dyn StatusObserver> {
        Arc::new(TelegramNotifier::new(
            self.configuration.clone(),
            self.outbox.clone(),
        ))
    }

    pub fn outbox(&self) -> Arc<Outbox> {
        self.outbox.clone()
    }
}

impl Feature for TelegramFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
    }

    fn loops(&self) -> Vec<Loop> {
        vec![Box::pin(send_forever(
            self.configuration.clone(),
            self.outbox.clone(),
            self.bot.clone(),
        ))]
    }
}
