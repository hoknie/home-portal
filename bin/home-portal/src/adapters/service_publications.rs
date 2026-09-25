use std::sync::Arc;

use portal_model::Environment;
use portal_proxy::{PublishedService, PublishedServices};
use portal_services::{ServiceEntry, ServicesFeature};

pub struct ServicePublications {
    pub services: Arc<ServicesFeature>,
}

impl ServicePublications {
    pub fn of(entries: Vec<ServiceEntry>, host: &Environment) -> Vec<PublishedService> {
        entries
            .into_iter()
            .filter_map(|entry| {
                let publication = entry.proxy.clone()?;
                let upstream = publication
                    .upstream
                    .clone()
                    .unwrap_or_else(|| entry.probe_address(host).to_string());
                Some(PublishedService {
                    id: entry.id,
                    upstream,
                    publication,
                })
            })
            .collect()
    }
}

impl PublishedServices for ServicePublications {
    fn published(&self) -> Vec<PublishedService> {
        Self::of(self.services.entries(), self.services.host())
    }
}
