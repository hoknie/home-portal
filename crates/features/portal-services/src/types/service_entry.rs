use std::collections::BTreeMap;

use portal_model::{Environment, Publication};
use serde::{Deserialize, Serialize};

use super::{ProbeSettings, ServiceLink};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceEntry {
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

impl ServiceEntry {
    pub const MAXIMUM_NAME_LENGTH: usize = 80;
    pub const MAXIMUM_DESCRIPTION_LENGTH: usize = 200;
    pub const MAXIMUM_LINKS: usize = 20;
    pub const MAXIMUM_NOTES_LENGTH: usize = 10_000;

    pub fn new(id: &str, name: &str, url: &str) -> ServiceEntry {
        ServiceEntry {
            id: id.to_string(),
            name: name.to_string(),
            url: url.to_string(),
            group: None,
            icon: None,
            description: None,
            addresses: BTreeMap::new(),
            environments: None,
            public: false,
            public_status: false,
            notify: None,
            links: Vec::new(),
            notes: None,
            widgets: Vec::new(),
            probe: ProbeSettings::default(),
            proxy: None,
        }
    }

    pub fn address_for(&self, environment: &Environment) -> &str {
        self.addresses
            .get(environment.as_str())
            .unwrap_or(&self.url)
    }

    pub fn shown_address(&self, environment: &Environment, publishing: Option<u16>) -> String {
        match (&self.proxy, publishing) {
            (Some(publication), Some(https_port)) if publication.published_in(environment) => {
                publication.address_on(https_port)
            }
            _ => self.address_for(environment).to_string(),
        }
    }

    pub fn probe_address(&self, host: &Environment) -> &str {
        match &self.probe.environment {
            Some(name) => self.addresses.get(name).unwrap_or(&self.url),
            None => self.address_for(host),
        }
    }

    pub fn visible_to(&self, environment: &Environment) -> bool {
        match &self.environments {
            None => true,
            Some(names) => names.iter().any(|name| name == environment.as_str()),
        }
    }

    pub fn public_in(&self, environment: &Environment) -> bool {
        self.public && self.visible_to(environment)
    }

    pub fn notifies(&self) -> bool {
        self.notify.unwrap_or(true)
    }

    pub fn probes_like(&self, other: &ServiceEntry) -> bool {
        self.id == other.id
            && self.url == other.url
            && self.addresses == other.addresses
            && self.probe == other.probe
    }
}
