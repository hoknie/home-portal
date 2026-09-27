use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::{ApiError, EventName, EventSink, PortalEvent};
use time::OffsetDateTime;

use crate::repositories::{origin, position, published_elsewhere, replace};
use crate::services::Showcase;
use crate::types::{ServiceEntry, ShownService};

#[derive(Clone)]
pub struct ChangeService {
    configuration: Arc<ConfigStore>,
    showcase: Showcase,
    events: Arc<dyn EventSink>,
}

impl ChangeService {
    pub fn new(
        configuration: Arc<ConfigStore>,
        showcase: Showcase,
        events: Arc<dyn EventSink>,
    ) -> ChangeService {
        ChangeService {
            configuration,
            showcase,
            events,
        }
    }

    pub async fn run(
        &self,
        id: &str,
        entry: ServiceEntry,
        revision: &Revision,
        user: &str,
    ) -> Result<Revisioned<ShownService>, ApiError> {
        let entry = self
            .showcase
            .checked(entry, &self.configuration.read().document)?;
        let target = origin(&self.configuration.read(), id)
            .ok_or(ApiError::NotFound(ServiceEntry::UNKNOWN))?;
        let (_, snapshot) = self
            .configuration
            .update(&target, revision, |document| {
                let index =
                    position(document, id).ok_or(ApiError::NotFound(ServiceEntry::UNKNOWN))?;
                if entry.id != id && position(document, &entry.id).is_some() {
                    return Err(ApiError::invalid("id", ServiceEntry::TAKEN_ID));
                }
                if published_elsewhere(document, &entry, id) {
                    return Err(ApiError::invalid("proxy.host", ServiceEntry::TAKEN_HOST));
                }
                replace(document, index, &entry);
                Ok(())
            })
            .await?;
        if entry.id != id {
            self.showcase.board.rename(id, &entry.id);
        }
        self.showcase.supervisor.reconcile();
        self.events.emit(PortalEvent::of(
            EventName::ServiceUpdated,
            OffsetDateTime::now_utc(),
            &[
                ("service.id", &entry.id),
                ("service.name", &entry.name),
                ("service.previous_id", id),
                ("user.name", user),
            ],
        ));
        Ok(Revisioned::new(
            self.showcase.shown(entry),
            snapshot.revision,
        ))
    }
}
