use std::sync::Arc;

use portal_config::{ConfigStore, deserialize_section};
use serde::Deserialize;

use crate::ports::PublishedServices;
use crate::types::{PublishedService, RawPublishedService};

#[derive(Deserialize, Default)]
struct Entries {
    #[serde(default)]
    services: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    url: String,
    #[serde(flatten)]
    published: RawPublishedService,
}

pub struct Catalogue {
    pub configuration: Arc<ConfigStore>,
}

impl PublishedServices for Catalogue {
    fn published(&self) -> Vec<PublishedService> {
        deserialize_section::<Entries>(&self.configuration.read().document)
            .unwrap_or_default()
            .services
            .into_iter()
            .filter_map(|entry| {
                let publication = entry.published.proxy?;
                Some(PublishedService {
                    id: entry.published.id,
                    upstream: publication.upstream.clone().unwrap_or(entry.url),
                    publication,
                })
            })
            .collect()
    }
}
