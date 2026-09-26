use std::net::IpAddr;

use ipnet::IpNet;
use portal_model::{Environment, Environments};
use toml_edit::DocumentMut;

use crate::services::{build, read_settings};
use crate::types::{DnsSettings, PublishedHost, ZoneBook};

pub fn environment(name: &str) -> Environment {
    Environment::parse(name).unwrap()
}

pub fn environments() -> Environments {
    Environments::new(vec![
        (
            environment("local"),
            vec![
                "192.168.1.0/24".parse::<IpNet>().unwrap(),
                "fd00::/64".parse().unwrap(),
            ],
        ),
        (
            environment("vpn"),
            vec!["10.8.0.0/24".parse::<IpNet>().unwrap()],
        ),
    ])
}

pub fn settings(extra: &str) -> DnsSettings {
    let document: DocumentMut = format!(
        "[environments.local]\nnetworks = [\"192.168.1.0/24\", \"fd00::/64\"]\n[environments.vpn]\nnetworks = [\"10.8.0.0/24\"]\n[proxy]\nenabled = true\nportal_host = \"portal.home\"\n{extra}"
    )
    .parse()
    .unwrap();
    read_settings(&document).unwrap()
}

pub fn published(host: &str, shown: Option<&[&str]>) -> PublishedHost {
    PublishedHost {
        host: host.to_string(),
        environments: shown.map(|names| names.iter().map(|name| environment(name)).collect()),
    }
}

pub fn book(extra: &str, hosts: &[PublishedHost], interfaces: &[&str]) -> ZoneBook {
    let interfaces: Vec<IpAddr> = interfaces
        .iter()
        .map(|text| text.parse().unwrap())
        .collect();
    build(&settings(extra), hosts, &interfaces, environments())
}
