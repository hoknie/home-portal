use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;

use portal_model::{Environment, Environments};

use crate::types::{DnsSettings, ProxyAddresses, PublishedHost, RecordData, Zone, ZoneBook};

pub fn build(
    settings: &DnsSettings,
    published: &[PublishedHost],
    interfaces: &[IpAddr],
    environments: Environments,
) -> ZoneBook {
    let named: Vec<Environment> = environments
        .named()
        .iter()
        .map(|(environment, _)| environment.clone())
        .collect();
    let addresses: BTreeMap<Environment, ProxyAddresses> = environments
        .named()
        .iter()
        .map(|(environment, networks)| {
            let found = settings
                .addresses
                .get(environment)
                .cloned()
                .unwrap_or_else(|| ProxyAddresses {
                    v4: interfaces.iter().find_map(|address| match address {
                        IpAddr::V4(v4)
                            if networks.iter().any(|network| network.contains(address)) =>
                        {
                            Some(*v4)
                        }
                        _ => None,
                    }),
                    v6: interfaces.iter().find_map(|address| match address {
                        IpAddr::V6(v6)
                            if networks.iter().any(|network| network.contains(address)) =>
                        {
                            Some(*v6)
                        }
                        _ => None,
                    }),
                });
            (environment.clone(), found)
        })
        .collect();
    let unaddressed = addresses
        .iter()
        .filter(|(_, found)| found.v4.is_none() && found.v6.is_none())
        .map(|(environment, _)| environment.clone())
        .collect();
    let mut zones: Vec<Zone> = settings
        .zones
        .iter()
        .map(|apex| Zone {
            apex: apex.clone(),
            single: false,
        })
        .collect();
    let mut names: BTreeMap<String, BTreeMap<Environment, Vec<RecordData>>> = BTreeMap::new();
    let mut hosts: Vec<(String, Vec<Environment>)> = Vec::new();
    if settings.proxy.enabled {
        if let Some(portal) = &settings.proxy.portal_host {
            hosts.push((portal.clone(), named.clone()));
        }
        for host in published {
            let shown = host.environments.clone().unwrap_or_else(|| named.clone());
            hosts.push((host.host.to_ascii_lowercase(), shown));
        }
    }
    for (host, shown) in &hosts {
        if !zones.iter().any(|zone| zone.holds(host)) {
            zones.push(Zone {
                apex: host.clone(),
                single: true,
            });
        }
        let entry = names.entry(host.clone()).or_default();
        for environment in shown
            .iter()
            .filter(|environment| named.contains(environment))
        {
            let found = addresses.get(environment).cloned().unwrap_or_default();
            let records = entry.entry(environment.clone()).or_default();
            records.extend(found.v4.map(RecordData::A));
            records.extend(found.v6.map(RecordData::Aaaa));
        }
    }
    for record in &settings.records {
        let entry = names.entry(record.name.clone()).or_default();
        for environment in record.environments.as_ref().unwrap_or(&named) {
            entry
                .entry(environment.clone())
                .or_default()
                .push(record.data.clone());
        }
    }
    let serial = serial_of(&zones, &names);
    ZoneBook {
        zones,
        names,
        addresses,
        unaddressed,
        environments,
        nameserver: settings.secure_host().map(str::to_string),
        ttl: settings.ttl,
        serial,
    }
}

fn serial_of(
    zones: &[Zone],
    names: &BTreeMap<String, BTreeMap<Environment, Vec<RecordData>>>,
) -> u32 {
    let mut hasher = DefaultHasher::new();
    zones.hash(&mut hasher);
    names.hash(&mut hasher);
    let hash = hasher.finish();
    (hash as u32) ^ ((hash >> 32) as u32)
}
