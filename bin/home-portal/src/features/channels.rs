use std::sync::Arc;

use portal_feature::Channel;
use portal_telegram::TelegramChannel;

use crate::types::BootError;

pub fn channels(telegram_endpoint: &str) -> Result<Vec<Arc<dyn Channel>>, BootError> {
    let telegram =
        TelegramChannel::new(telegram_endpoint).map_err(|message| BootError::Feature {
            name: TelegramChannel::NAME,
            message,
        })?;
    Ok(vec![Arc::new(telegram)])
}
