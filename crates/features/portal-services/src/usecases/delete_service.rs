use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::{ApiError, EventName, EventSink, PortalEvent};
use portal_model::Environment;
use time::OffsetDateTime;

use crate::repositories::{origin, position, remove};
use crate::services::Showcase;
use crate::types::{ServiceEntry, ServicesSection, ShownService};

#[derive(Clone)]
pub struct DeleteService {
    configuration: Arc<ConfigStore>,
    showcase: Showcase,
    events: Arc<dyn EventSink>,
}

impl DeleteService {
    pub fn new(
        configuration: Arc<ConfigStore>,
        showcase: Showcase,
        events: Arc<dyn EventSink>,
    ) -> DeleteService {
        DeleteService {
            configuration,
            showcase,
            events,
        }
    }

    pub async fn run(
        &self,
        id: &str,
        revision: &Revision,
        environment: &Environment,
        user: &str,
    ) -> Result<Revisioned<Vec<ShownService>>, ApiError> {
        let name = ServicesSection::read(&self.configuration.read().document)
            .ok()
            .and_then(|section| section.services.into_iter().find(|entry| entry.id == id))
            .map(|entry| entry.name)
            .unwrap_or_default();
        let target = origin(&self.configuration.read(), id)
            .ok_or(ApiError::NotFound(ServiceEntry::UNKNOWN))?;
        let (_, snapshot) = self
            .configuration
            .update(&target, revision, |document| {
                let index =
                    position(document, id).ok_or(ApiError::NotFound(ServiceEntry::UNKNOWN))?;
                remove(document, index);
                Ok(())
            })
            .await?;
        self.showcase.supervisor.reconcile();
        self.events.emit(PortalEvent::of(
            EventName::ServiceDeleted,
            OffsetDateTime::now_utc(),
            &[
                ("service.id", id),
                ("service.name", &name),
                ("user.name", user),
            ],
        ));
        let services = self.showcase.listed(&snapshot.document, environment)?;
        Ok(Revisioned::new(services, snapshot.revision))
    }
}
