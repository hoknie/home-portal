use std::sync::Arc;

use portal_icons::IconsFeature;
use portal_model::Environment;
use portal_public::{PublicService, PublicServices};
use portal_services::ServicesFeature;

pub struct ServiceCatalogue {
    pub services: Arc<ServicesFeature>,
    pub icons: Arc<IconsFeature>,
}

impl PublicServices for ServiceCatalogue {
    fn public_services(&self, environment: &Environment) -> Vec<PublicService> {
        self.services
            .entries()
            .into_iter()
            .filter(|entry| entry.public_in(environment))
            .map(|entry| PublicService {
                icon: self
                    .icons
                    .icons()
                    .icon_of(&entry.id)
                    .map(|_| format!("/api/public/icons/{}", entry.id)),
                address: entry.shown_address(environment, self.services.publishing()),
                status: entry
                    .public_status
                    .then(|| self.services.status_of(&entry.id)),
                id: entry.id,
                name: entry.name,
                group: entry.group,
                description: entry.description,
            })
            .collect()
    }
}
