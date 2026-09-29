use portal_config::{Revision, Revisioned, Section};
use portal_feature::ApiError;

use super::ListNotifications;
use crate::repositories::{rules_origin, write_rules};
use crate::services::ChannelSet;
use crate::types::{NotificationsView, Rules};

#[derive(Clone)]
pub struct ChangeRules {
    channels: ChannelSet,
    list: ListNotifications,
}

impl ChangeRules {
    pub fn new(channels: ChannelSet, list: ListNotifications) -> ChangeRules {
        ChangeRules { channels, list }
    }

    pub async fn run(
        &self,
        rules: Rules,
        revision: &Revision,
    ) -> Result<Revisioned<NotificationsView>, ApiError> {
        let configuration = &self.channels.configuration;
        let target = rules_origin(&configuration.read())
            .unwrap_or_else(|| configuration.home_of(Section::Notifications));
        let (_, snapshot) = configuration
            .update(&target, revision, |document| {
                write_rules(document, &rules);
                let problems = Rules::problems(document);
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
