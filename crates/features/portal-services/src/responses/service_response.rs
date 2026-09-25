use std::collections::BTreeMap;

use portal_model::{Publication, ServiceStatus};
use serde::{Deserialize, Serialize};

use crate::types::{ProbeSettings, ServiceEntry, ServiceLink, Viewpoint};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub id: String,
    pub name: String,
    pub url: String,
    pub address: String,
    pub probe_address: String,
    pub addresses: BTreeMap<String, String>,
    pub environments: Option<Vec<String>>,
    pub group: Option<String>,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub public: bool,
    pub public_status: bool,
    pub notify: bool,
    pub links: Vec<ServiceLink>,
    pub notes: Option<String>,
    pub widgets: Vec<String>,
    pub probe: ProbeSettings,
    pub proxy: Option<Publication>,
    pub status: ServiceStatus,
}

impl ServiceResponse {
    pub fn of(
        entry: ServiceEntry,
        viewpoint: Viewpoint<'_>,
        status: ServiceStatus,
    ) -> ServiceResponse {
        ServiceResponse {
            address: entry.shown_address(viewpoint.environment, viewpoint.publishing),
            probe_address: entry.probe_address(viewpoint.host).to_string(),
            notify: entry.notifies(),
            id: entry.id,
            name: entry.name,
            url: entry.url,
            addresses: entry.addresses,
            environments: entry.environments,
            group: entry.group,
            icon: entry.icon,
            description: entry.description,
            public: entry.public,
            public_status: entry.public_status,
            links: entry.links,
            notes: entry.notes,
            widgets: entry.widgets,
            probe: entry.probe,
            proxy: entry.proxy,
            status,
        }
    }
}
