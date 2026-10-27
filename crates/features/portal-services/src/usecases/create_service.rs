use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned, Section};
use portal_feature::{ApiError, EventName, EventSink, PortalEvent};
use time::OffsetDateTime;

use crate::repositories::{append, host_taken, position};
use crate::services::Showcase;
use crate::types::{ServiceEntry, ShownService};

#[derive(Clone)]
pub struct CreateService {
    configuration: Arc<ConfigStore>,
    showcase: Showcase,
    events: Arc<dyn EventSink>,
}

impl CreateService {
    pub fn new(
        configuration: Arc<ConfigStore>,
        showcase: Showcase,
        events: Arc<dyn EventSink>,
    ) -> CreateService {
        CreateService {
            configuration,
            showcase,
            events,
        }
    }

    pub async fn run(
        &self,
        entry: ServiceEntry,
        revision: &Revision,
        user: &str,
    ) -> Result<Revisioned<ShownService>, ApiError> {
        let entry = self
            .showcase
            .checked(entry, &self.configuration.read().document)?;
        let target = self.configuration.home_of(Section::Services);
        let (_, snapshot) = self
            .configuration
            .update(&target, revision, |document| {
                if position(document, &entry.id).is_some() {
                    return Err(ApiError::invalid("id", ServiceEntry::TAKEN_ID));
                }
                if let Some(publication) = &entry.proxy
                    && host_taken(document, &publication.host, &entry.id)
                {
                    return Err(ApiError::invalid("proxy.host", ServiceEntry::TAKEN_HOST));
                }
                append(document, &entry.normalized());
                Ok(())
            })
            .await?;
        self.showcase.supervisor.reconcile();
        self.events.emit(PortalEvent::of(
            EventName::ServiceCreated,
            OffsetDateTime::now_utc(),
            &[
                ("service.id", &entry.id),
                ("service.name", &entry.name),
                ("user.name", user),
            ],
        ));
        Ok(Revisioned::new(
            self.showcase.shown(entry),
            snapshot.revision,
        ))
    }
}
