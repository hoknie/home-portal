use portal_config::{Revision, Revisioned, Section};
use portal_feature::{ApiError, FieldError};
use serde_json::Value;

use super::ListNotifications;
use crate::repositories::{channel_origin, channel_table};
use crate::services::ChannelSet;
use crate::types::NotificationsView;

#[derive(Clone)]
pub struct ChangeChannel {
    channels: ChannelSet,
    list: ListNotifications,
}

impl ChangeChannel {
    pub const UNKNOWN: &'static str = "no such notification channel";

    pub fn new(channels: ChannelSet, list: ListNotifications) -> ChangeChannel {
        ChangeChannel { channels, list }
    }

    pub async fn run(
        &self,
        name: &str,
        settings: &Value,
        revision: &Revision,
    ) -> Result<Revisioned<NotificationsView>, ApiError> {
        let entry = self
            .channels
            .find(name)
            .ok_or(ApiError::NotFound(Self::UNKNOWN))?;
        let configuration = &self.channels.configuration;
        let target = channel_origin(&configuration.read(), name)
            .unwrap_or_else(|| configuration.home_of(Section::Notifications));
        let channel = entry.channel.clone();
        let (_, snapshot) = configuration
            .update(&target, revision, |document| {
                channel
                    .apply(channel_table(document, name), settings)
                    .map_err(ApiError::Invalid)?;
                let prefix = format!("notifications.{name}.");
                let problems: Vec<FieldError> = channel
                    .problems(document, configuration.as_ref())
                    .into_iter()
                    .map(|problem| {
                        FieldError::new(problem.field.trim_start_matches(&prefix), problem.message)
                    })
                    .collect();
                if problems.is_empty() {
                    Ok(())
                } else {
                    Err(ApiError::Invalid(problems))
                }
            })
            .await?;
        Ok(Revisioned::new(
            self.list.view_of(&snapshot),
            snapshot.revision.clone(),
        ))
    }
}
