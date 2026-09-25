use std::net::{IpAddr, Ipv4Addr};

use ipnet::IpNet;

pub const LOOPBACK: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

pub fn covers_loopback<'a>(networks: impl IntoIterator<Item = &'a str>) -> bool {
    networks
        .into_iter()
        .filter_map(|text| {
            let text = text.trim();
            text.parse::<IpNet>()
                .ok()
                .or_else(|| text.parse::<IpAddr>().ok().map(IpNet::from))
        })
        .any(|network| network.contains(&LOOPBACK))
}
