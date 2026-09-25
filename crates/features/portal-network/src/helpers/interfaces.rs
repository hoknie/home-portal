use std::collections::BTreeMap;

use crate::responses::InterfaceResponse;

pub fn host_interfaces() -> Vec<InterfaceResponse> {
    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for interface in if_addrs::get_if_addrs().unwrap_or_default() {
        grouped
            .entry(interface.name.clone())
            .or_default()
            .push(interface.ip().to_string());
    }
    grouped
        .into_iter()
        .map(|(name, addresses)| InterfaceResponse { name, addresses })
        .collect()
}
