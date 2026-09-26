use std::net::IpAddr;
use std::sync::Arc;

use portal_config::ConfigStore;
use portal_dns::{DnsSources, PublishedHost};
use portal_model::{Environment, Environments};
use portal_network::{host_interfaces, read_environments};
use portal_services::ServicesFeature;

pub struct DnsDirectory {
    pub configuration: Arc<ConfigStore>,
    pub services: Arc<ServicesFeature>,
}

impl DnsSources for DnsDirectory {
    fn environments(&self) -> Environments {
        read_environments(&self.configuration.read().document).unwrap_or_default()
    }

    fn published(&self) -> Vec<PublishedHost> {
        self.services
            .entries()
            .into_iter()
            .filter_map(|entry| {
                let publication = entry.proxy?;
                Some(PublishedHost {
                    host: publication.host,
                    environments: entry.environments.map(|names| {
                        names
                            .iter()
                            .filter_map(|name| Environment::parse(name).ok())
                            .collect()
                    }),
                })
            })
            .collect()
    }

    fn interfaces(&self) -> Vec<IpAddr> {
        host_interfaces()
            .into_iter()
            .flat_map(|interface| interface.addresses)
            .filter_map(|address| address.parse().ok())
            .collect()
    }
}
