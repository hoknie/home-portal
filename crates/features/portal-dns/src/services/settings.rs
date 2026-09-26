use std::collections::BTreeMap;
use std::net::IpAddr;
use std::path::PathBuf;

use portal_config::deserialize_section;
use portal_feature::FieldError;
use portal_model::Environment;
use toml_edit::DocumentMut;

use crate::helpers::{inside, normalized};
use crate::types::{
    DnsHttps, DnsRecord, DnsSettings, DnsTls, ProxyAddresses, ProxyView, RawAddresses, RawDns,
    RawDnsSection, RawRecord, RecordData, RecordKind,
};

pub const SECTION: &str = "dns";
pub const NOT_AN_ADDRESS: &str = "must be an IP address";
pub const PORT_RANGE: &str = "must be between 1 and 65535";
pub const NOT_A_NAME: &str = "must be a domain name such as home or lan.example.com";
pub const TTL_RANGE: &str = "must be between 5 and 86400 seconds";
pub const UNKNOWN_ENVIRONMENT: &str = "must name a configured environment other than internet";
pub const TOO_MANY_ADDRESSES: &str = "must be at most one IPv4 and one IPv6 address";
pub const OUTSIDE_ZONES: &str = "must lie inside one of dns.zones";
pub const UNKNOWN_KIND: &str = "must be A, AAAA, CNAME or TXT";
pub const BAD_VALUE: &str = "does not fit the record type";
pub const SAME_PORT: &str = "must differ from dns.port";
pub const TOGETHER: &str = "must be given together with dns.tls.certificate";
pub const NEEDS_PROXY: &str = "needs the proxy to be enabled";
pub const LONGEST_TEXT: usize = 255;

pub fn read_settings(document: &DocumentMut) -> Result<DnsSettings, Vec<FieldError>> {
    let section: RawDnsSection =
        deserialize_section(document).map_err(|message| vec![FieldError::new(SECTION, message)])?;
    let mut errors = Vec::new();
    let environments: Vec<Environment> = section
        .environments
        .keys()
        .filter_map(|name| Environment::parse(name).ok())
        .filter(|environment| !environment.is_internet())
        .collect();
    let proxy = ProxyView {
        enabled: section.proxy.enabled.unwrap_or(false),
        managed: section.proxy.managed.unwrap_or(false),
        portal_host: section.proxy.portal_host.as_deref().and_then(normalized),
        https_port: section
            .proxy
            .https_port
            .and_then(|port| u16::try_from(port).ok()),
        tls: section.proxy.tls.clone().unwrap_or_default(),
    };
    let settings = check(&section.dns, &environments, proxy, &mut errors);
    if errors.is_empty() {
        Ok(settings)
    } else {
        Err(errors)
    }
}

pub fn validate_dns(document: &DocumentMut) -> Vec<FieldError> {
    read_settings(document).err().unwrap_or_default()
}

fn field(name: &str) -> String {
    format!("{SECTION}.{name}")
}

fn port(value: Option<i64>, default: u16, name: &str, errors: &mut Vec<FieldError>) -> u16 {
    match value {
        None => default,
        Some(value) => u16::try_from(value)
            .ok()
            .filter(|port| *port != 0)
            .unwrap_or_else(|| {
                errors.push(FieldError::new(field(name), PORT_RANGE));
                default
            }),
    }
}

fn check(
    raw: &RawDns,
    environments: &[Environment],
    proxy: ProxyView,
    errors: &mut Vec<FieldError>,
) -> DnsSettings {
    let defaults = DnsSettings::default();
    let address = match raw.address.as_deref() {
        None => defaults.address,
        Some(text) => text.trim().parse().unwrap_or_else(|_| {
            errors.push(FieldError::new(field("address"), NOT_AN_ADDRESS));
            defaults.address
        }),
    };
    let dns_port = port(raw.port, DnsSettings::DEFAULT_PORT, "port", errors);
    let zones: Vec<String> = raw
        .zones
        .iter()
        .enumerate()
        .filter_map(|(index, zone)| {
            let found = normalized(zone);
            if found.is_none() {
                errors.push(FieldError::new(
                    field(&format!("zones[{index}]")),
                    NOT_A_NAME,
                ));
            }
            found
        })
        .collect();
    let ttl = match raw.ttl {
        None => DnsSettings::DEFAULT_TTL,
        Some(ttl) => u32::try_from(ttl)
            .ok()
            .filter(|ttl| (DnsSettings::SHORTEST_TTL..=DnsSettings::LONGEST_TTL).contains(ttl))
            .unwrap_or_else(|| {
                errors.push(FieldError::new(field("ttl"), TTL_RANGE));
                DnsSettings::DEFAULT_TTL
            }),
    };
    let addresses = check_addresses(&raw.addresses, environments, errors);
    let records = raw
        .records
        .iter()
        .enumerate()
        .filter_map(|(index, record)| check_record(index, record, &zones, environments, errors))
        .collect();
    let tls_port = port(
        raw.tls.port,
        DnsSettings::DEFAULT_TLS_PORT,
        "tls.port",
        errors,
    );
    if tls_port == dns_port {
        errors.push(FieldError::new(field("tls.port"), SAME_PORT));
    }
    let present = |value: &Option<String>| {
        value
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(PathBuf::from)
    };
    let (certificate, key) = (present(&raw.tls.certificate), present(&raw.tls.key));
    if certificate.is_some() != key.is_some() {
        let name = if certificate.is_some() {
            "tls.key"
        } else {
            "tls.certificate"
        };
        errors.push(FieldError::new(field(name), TOGETHER));
    }
    let https = DnsHttps {
        enabled: raw.https.enabled.unwrap_or(false),
        host: raw.https.host.as_deref().and_then(|host| {
            let found = normalized(host);
            if found.is_none() {
                errors.push(FieldError::new(field("https.host"), NOT_A_NAME));
            }
            found
        }),
    };
    if https.enabled && !proxy.enabled {
        errors.push(FieldError::new(field("https.enabled"), NEEDS_PROXY));
    }
    DnsSettings {
        enabled: raw.enabled.unwrap_or(false),
        address,
        port: dns_port,
        zones,
        ttl,
        addresses,
        records,
        tls: DnsTls {
            enabled: raw.tls.enabled.unwrap_or(false),
            port: tls_port,
            certificate,
            key,
        },
        https,
        proxy,
    }
}

fn check_addresses(
    raw: &BTreeMap<String, RawAddresses>,
    environments: &[Environment],
    errors: &mut Vec<FieldError>,
) -> BTreeMap<Environment, ProxyAddresses> {
    let mut addresses = BTreeMap::new();
    for (name, value) in raw {
        let key = field(&format!("addresses.{name}"));
        let Some(environment) = environments.iter().find(|known| known.as_str() == name) else {
            errors.push(FieldError::new(key, UNKNOWN_ENVIRONMENT));
            continue;
        };
        let texts = match value {
            RawAddresses::One(text) => vec![text.clone()],
            RawAddresses::Many(texts) => texts.clone(),
        };
        let Ok(parsed) = texts
            .iter()
            .map(|text| text.trim().parse::<IpAddr>())
            .collect::<Result<Vec<IpAddr>, _>>()
        else {
            errors.push(FieldError::new(key, NOT_AN_ADDRESS));
            continue;
        };
        let v4: Vec<_> = parsed
            .iter()
            .filter_map(|address| match address {
                IpAddr::V4(v4) => Some(*v4),
                IpAddr::V6(_) => None,
            })
            .collect();
        let v6: Vec<_> = parsed
            .iter()
            .filter_map(|address| match address {
                IpAddr::V6(v6) => Some(*v6),
                IpAddr::V4(_) => None,
            })
            .collect();
        if v4.len() > 1 || v6.len() > 1 {
            errors.push(FieldError::new(key, TOO_MANY_ADDRESSES));
            continue;
        }
        let found = ProxyAddresses {
            v4: v4.first().copied(),
            v6: v6.first().copied(),
        };
        addresses.insert(environment.clone(), found);
    }
    addresses
}

fn check_record(
    index: usize,
    raw: &RawRecord,
    zones: &[String],
    environments: &[Environment],
    errors: &mut Vec<FieldError>,
) -> Option<DnsRecord> {
    let key = |name: &str| field(&format!("records[{index}].{name}"));
    let before = errors.len();
    let name = match normalized(&raw.name) {
        None => {
            errors.push(FieldError::new(key("name"), NOT_A_NAME));
            None
        }
        Some(name) if !zones.iter().any(|zone| inside(&name, zone)) => {
            errors.push(FieldError::new(key("name"), OUTSIDE_ZONES));
            None
        }
        Some(name) => Some(name),
    };
    let value = raw.value.trim();
    let data = match RecordKind::parse(raw.kind.trim()) {
        None => {
            errors.push(FieldError::new(key("type"), UNKNOWN_KIND));
            None
        }
        Some(kind) => {
            let data = match kind {
                RecordKind::A => value.parse().ok().map(RecordData::A),
                RecordKind::Aaaa => value.parse().ok().map(RecordData::Aaaa),
                RecordKind::Cname => normalized(value).map(RecordData::Cname),
                _ => (!value.is_empty() && value.len() <= LONGEST_TEXT)
                    .then(|| RecordData::Txt(value.to_string())),
            };
            if data.is_none() {
                errors.push(FieldError::new(key("value"), BAD_VALUE));
            }
            data
        }
    };
    let chosen = raw.environments.as_ref().map(|names| {
        names
            .iter()
            .filter_map(|name| {
                let found = environments
                    .iter()
                    .find(|known| known.as_str() == name)
                    .cloned();
                if found.is_none() {
                    errors.push(FieldError::new(key("environments"), UNKNOWN_ENVIRONMENT));
                }
                found
            })
            .collect()
    });
    (errors.len() == before).then(|| DnsRecord {
        name: name.unwrap_or_default(),
        data: data.unwrap_or(RecordData::Txt(String::new())),
        environments: chosen,
    })
}
