use std::sync::Arc;

use portal_config::{ConfigStore, Storage};
use time::OffsetDateTime;
use url::Url;

use super::IconCache;
use crate::clients::{Fetched, Fetcher, discover_icon};
use crate::helpers::sniff;
use crate::types::{IconSource, IconState, PreviewedIcon, StoredIcon};

pub struct Icons {
    configuration: Arc<ConfigStore>,
    cache: IconCache,
    fetcher: Fetcher,
    catalog: String,
}

pub const CATALOG: &str = "https://cdn.jsdelivr.net/gh/homarr-labs/dashboard-icons/png";

impl Icons {
    pub fn new(configuration: Arc<ConfigStore>, catalog: &str) -> Result<Icons, String> {
        let cache = IconCache::open(configuration.storage(Storage::Icons));
        Ok(Icons {
            configuration,
            cache,
            fetcher: Fetcher::new()?,
            catalog: catalog.trim_end_matches('/').to_string(),
        })
    }

    pub fn cache(&self) -> &IconCache {
        &self.cache
    }

    pub fn icon_of(&self, service: &str) -> Option<StoredIcon> {
        self.cache.read(service)
    }

    pub fn public_icon_of(&self, service: &str, environment: &str) -> Option<StoredIcon> {
        self.is_public(service, environment)
            .then(|| self.cache.read(service))
            .flatten()
    }

    fn is_public(&self, service: &str, environment: &str) -> bool {
        let snapshot = self.configuration.read();
        let Some(entries) = snapshot
            .document
            .get("services")
            .and_then(|item| item.as_array_of_tables())
        else {
            return false;
        };
        entries
            .iter()
            .find(|table| table.get("id").and_then(|item| item.as_str()) == Some(service))
            .is_some_and(|table| {
                let public = table
                    .get("public")
                    .and_then(|item| item.as_bool())
                    .unwrap_or(false);
                let visible = match table.get("environments").and_then(|item| item.as_array()) {
                    None => true,
                    Some(names) => names.iter().any(|name| name.as_str() == Some(environment)),
                };
                public && visible
            })
    }

    pub fn described(&self) -> Vec<IconState> {
        self.services()
            .into_iter()
            .map(|(service, source)| {
                let entry = self.cache.entry(&service);
                IconState {
                    available: entry.is_some(),
                    fetched_at: entry.as_ref().map(|entry| entry.fetched_at),
                    problem: None,
                    service,
                    source,
                }
            })
            .collect()
    }

    pub async fn refresh_all(&self, now: OffsetDateTime) {
        for (service, source) in self.services() {
            if let Err(problem) = self.refresh(&service, &source, now).await {
                tracing::debug!(service, %problem, "the icon was not refreshed");
            }
        }
    }

    pub async fn refresh(
        &self,
        service: &str,
        source: &str,
        now: OffsetDateTime,
    ) -> Result<(), String> {
        let parsed = IconSource::parse(source).map_err(|problem| problem.to_string())?;
        if !parsed.is_fetched() || !self.cache.stale(service, source, now) {
            return Ok(());
        }
        let address = self.service_address(service);
        let fetched = self.bytes_of(&parsed, address.as_ref()).await?;
        let content_type = sniff(&fetched.bytes, fetched.content_type.as_deref())
            .ok_or_else(|| "what was fetched is not an image the portal serves".to_string())?;
        self.cache
            .store(service, source, &fetched.bytes, &content_type, now)
    }

    pub async fn preview(
        &self,
        source: &IconSource,
        address: Option<&Url>,
    ) -> Result<PreviewedIcon, String> {
        let fetched = self.bytes_of(source, address).await?;
        let content_type = sniff(&fetched.bytes, fetched.content_type.as_deref())
            .ok_or_else(|| "what was fetched is not an image the portal serves".to_string())?;
        Ok(PreviewedIcon {
            bytes: fetched.bytes,
            content_type,
        })
    }

    async fn bytes_of(
        &self,
        source: &IconSource,
        address: Option<&Url>,
    ) -> Result<Fetched, String> {
        match source {
            IconSource::File(path) => Ok(Fetched {
                bytes: std::fs::read(path).map_err(|error| format!("{path}: {error}"))?,
                content_type: mime_guess::from_path(path)
                    .first()
                    .map(|kind| kind.to_string()),
            }),
            IconSource::Address(url) => self.fetcher.get(url, Fetcher::ICON_CEILING_BYTES).await,
            IconSource::Catalog(slug) => {
                let url = Url::parse(&format!("{}/{slug}.png", self.catalog))
                    .map_err(|error| error.to_string())?;
                self.fetcher.get(&url, Fetcher::ICON_CEILING_BYTES).await
            }
            IconSource::Discovered => {
                let address =
                    address.ok_or_else(|| "the service has no address to look at".to_string())?;
                let found = discover_icon(&self.fetcher, address).await?;
                self.fetcher.get(&found, Fetcher::ICON_CEILING_BYTES).await
            }
            IconSource::Lucide(_) => Err("a lucide icon is drawn by the interface".to_string()),
        }
    }

    fn services(&self) -> Vec<(String, String)> {
        let snapshot = self.configuration.read();
        snapshot
            .document
            .get("services")
            .and_then(|item| item.as_array_of_tables())
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|table| {
                        let id = table.get("id")?.as_str()?.to_string();
                        let icon = table.get("icon")?.as_str()?.to_string();
                        Some((id, icon))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn service_address(&self, service: &str) -> Option<Url> {
        let snapshot = self.configuration.read();
        let entries = snapshot.document.get("services")?.as_array_of_tables()?;
        let table = entries
            .iter()
            .find(|table| table.get("id").and_then(|item| item.as_str()) == Some(service))?;
        Url::parse(table.get("url")?.as_str()?).ok()
    }
}
