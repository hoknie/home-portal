use portal_feature::{Module, ModuleSwitches};
use serde_json::{Map, Value, json};

use super::PortalService;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PortalState {
    pub services: Vec<PortalService>,
    pub address: String,
    pub port: u16,
    pub url: String,
    pub environments: Vec<String>,
}

impl PortalState {
    pub const SERVICE_FIELDS: [&'static str; 9] = [
        "id",
        "name",
        "group",
        "url",
        "address",
        "state",
        "since",
        "latency_milliseconds",
        "public",
    ];
    pub const NETWORK_FIELDS: [&'static str; 3] = ["address", "port", "url"];
    pub const MODULE_FIELDS: [&'static str; 1] = ["is_enabled"];

    pub fn value_with(&self, switches: ModuleSwitches) -> Value {
        let modules: Vec<(&str, bool)> = Module::ALL
            .iter()
            .map(|module| (module.name(), switches.is_on(*module)))
            .collect();
        self.value(&modules)
    }

    pub fn value(&self, modules: &[(&str, bool)]) -> Value {
        let services: Vec<Value> = self
            .services
            .iter()
            .map(|service| {
                json!({
                    "id": service.id,
                    "name": service.name,
                    "group": service.group,
                    "url": service.url,
                    "address": service.address,
                    "state": service.state,
                    "since": service.since,
                    "latency_milliseconds": service.latency_milliseconds,
                    "public": service.public,
                })
            })
            .collect();
        let modules: Map<String, Value> = modules
            .iter()
            .map(|(name, on)| (name.to_string(), json!({ "is_enabled": on })))
            .collect();
        json!({
            "services": services,
            "network": { "address": self.address, "port": self.port, "url": self.url },
            "modules": modules,
            "environments": self.environments,
        })
    }
}
