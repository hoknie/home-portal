use std::net::IpAddr;

use super::Environment;
use ipnet::IpNet;

#[derive(Debug, Clone, Default)]
pub struct Environments {
    named: Vec<(Environment, Vec<IpNet>)>,
}

impl Environments {
    pub fn new(named: Vec<(Environment, Vec<IpNet>)>) -> Environments {
        Environments { named }
    }

    pub fn of(&self, address: IpAddr) -> Environment {
        self.named
            .iter()
            .find(|(_, networks)| networks.iter().any(|network| network.contains(&address)))
            .map(|(environment, _)| environment.clone())
            .unwrap_or_else(Environment::internet)
    }

    pub fn named(&self) -> &[(Environment, Vec<IpNet>)] {
        &self.named
    }

    pub fn names(&self) -> Vec<Environment> {
        let mut names: Vec<Environment> = self
            .named
            .iter()
            .map(|(environment, _)| environment.clone())
            .collect();
        names.push(Environment::internet());
        names
    }

    pub fn knows(&self, environment: &Environment) -> bool {
        environment.is_internet() || self.named.iter().any(|(known, _)| known == environment)
    }

    pub fn effective(&self, detected: &Environment, chosen: Option<&str>) -> Environment {
        if detected.is_internet() {
            return detected.clone();
        }
        chosen
            .and_then(|name| Environment::parse(name).ok())
            .filter(|environment| self.knows(environment))
            .unwrap_or_else(|| detected.clone())
    }
}
