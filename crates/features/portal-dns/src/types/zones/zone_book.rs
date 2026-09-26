use std::collections::BTreeMap;
use std::net::IpAddr;

use portal_model::{Environment, Environments};

use super::Zone;
use crate::types::{ProxyAddresses, RecordData};

#[derive(Debug, Clone, Default)]
pub struct ZoneBook {
    pub zones: Vec<Zone>,
    pub names: BTreeMap<String, BTreeMap<Environment, Vec<RecordData>>>,
    pub addresses: BTreeMap<Environment, ProxyAddresses>,
    pub unaddressed: Vec<Environment>,
    pub environments: Environments,
    pub nameserver: Option<String>,
    pub ttl: u32,
    pub serial: u32,
}

impl ZoneBook {
    pub const MAILBOX_LABEL: &'static str = "hostmaster";
    pub const NAMESERVER_LABEL: &'static str = "ns";

    pub fn environment_of(&self, address: IpAddr) -> Environment {
        self.environments.of(address.to_canonical())
    }

    pub fn zone_of(&self, name: &str) -> Option<&Zone> {
        self.zones
            .iter()
            .filter(|zone| zone.holds(name))
            .max_by_key(|zone| zone.apex.len())
    }

    pub fn nameserver_of(&self, zone: &Zone) -> String {
        self.nameserver
            .clone()
            .unwrap_or_else(|| format!("{}.{}", Self::NAMESERVER_LABEL, zone.apex))
    }

    pub fn records(&self, name: &str, environment: &Environment) -> Option<&[RecordData]> {
        self.names
            .get(name)
            .and_then(|environments| environments.get(environment))
            .map(Vec::as_slice)
            .filter(|records| !records.is_empty())
    }
}
