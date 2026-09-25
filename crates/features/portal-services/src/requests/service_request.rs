use std::collections::BTreeMap;

use portal_model::Publication;
use serde::Deserialize;

use crate::types::{ProbeSettings, ServiceEntry, ServiceLink};

#[derive(Debug, Deserialize)]
pub struct ServiceRequest {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub addresses: BTreeMap<String, String>,
    #[serde(default)]
    pub environments: Option<Vec<String>>,
    #[serde(default)]
    pub public: bool,
    #[serde(default)]
    pub public_status: bool,
    #[serde(default)]
    pub notify: Option<bool>,
    #[serde(default)]
    pub links: Vec<ServiceLink>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub widgets: Vec<String>,
    #[serde(default)]
    pub probe: ProbeSettings,
    #[serde(default)]
    pub proxy: Option<Publication>,
}

impl ServiceRequest {
    pub fn into_entry(self) -> ServiceEntry {
        let blank_to_none = |text: Option<String>| {
            text.map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty())
        };
        ServiceEntry {
            id: self.id.trim().to_string(),
            name: self.name.trim().to_string(),
            url: self.url.trim().to_string(),
            group: blank_to_none(self.group),
            icon: blank_to_none(self.icon),
            description: blank_to_none(self.description),
            addresses: self
                .addresses
                .into_iter()
                .map(|(name, address)| (name, address.trim().to_string()))
                .filter(|(_, address)| !address.is_empty())
                .collect(),
            environments: self.environments,
            public: self.public,
            public_status: self.public_status,
            notify: self.notify,
            links: self
                .links
                .into_iter()
                .map(|link| ServiceLink {
                    title: link.title.trim().to_string(),
                    url: link.url.trim().to_string(),
                })
                .collect(),
            notes: self
                .notes
                .map(|notes| notes.trim_end().to_string())
                .filter(|notes| !notes.trim().is_empty()),
            widgets: self.widgets,
            probe: self.probe,
            proxy: self
                .proxy
                .map(Self::trimmed)
                .filter(|publication| !publication.host.is_empty()),
        }
    }

    fn trimmed(publication: Publication) -> Publication {
        let upstream = publication
            .upstream
            .map(|upstream| upstream.trim().to_string())
            .filter(|upstream| !upstream.is_empty());
        Publication {
            host: publication.host.trim().to_ascii_lowercase(),
            upstream,
            ..publication
        }
    }
}
