use std::sync::Arc;

use portal_config::ConfigStore;

use crate::clients::Bot;
use crate::services::Outbox;
use crate::types::TelegramSection;

pub async fn send_forever(configuration: Arc<ConfigStore>, outbox: Arc<Outbox>, bot: Arc<Bot>) {
    loop {
        let message = outbox.next().await;
        let settings = match TelegramSection::read(&configuration.read().document) {
            Ok(settings) => settings,
            Err(problem) => {
                tracing::warn!(%problem, "the telegram settings could not be read");
                continue;
            }
        };
        let Some(token) = settings.secret.and_then(|name| configuration.secret(&name)) else {
            tracing::warn!("a message was dropped because the telegram token is not set");
            continue;
        };
        if let Err(problem) = bot.send(&token, &message).await {
            tracing::warn!(%problem, "a telegram message was not delivered");
        }
    }
}
