use std::net::IpAddr;

use ipnet::IpNet;
use portal_config::deserialize_section;
use portal_feature::FieldError;
use toml_edit::DocumentMut;
use url::Url;

use crate::types::{NetworkSettings, RawNetwork, RawNetworkSection};

pub const SECTION_PREFIX: &str = "network.";

pub fn validate_network(document: &DocumentMut) -> Vec<FieldError> {
    match read_network(document) {
        Ok(_) => Vec::new(),
        Err(errors) => errors,
    }
}

pub fn read_network(document: &DocumentMut) -> Result<NetworkSettings, Vec<FieldError>> {
    let section: RawNetworkSection = deserialize_section(document)
        .map_err(|message| vec![FieldError::new("network", message)])?;
    check_network(&section.network).map_err(|errors| {
        errors
            .into_iter()
            .map(|error| error.prefixed(SECTION_PREFIX))
            .collect()
    })
}

pub fn check_network(raw: &RawNetwork) -> Result<NetworkSettings, Vec<FieldError>> {
    let mut errors = Vec::new();
    let mut settings = NetworkSettings::default();
    if let Some(address) = &raw.address {
        match address.trim().parse::<IpAddr>() {
            Ok(parsed) => settings.address = parsed,
            Err(_) => errors.push(FieldError::new(
                "address",
                "must be an IP address such as 0.0.0.0 or 192.168.1.10",
            )),
        }
    }
    if let Some(port) = raw.port {
        match u16::try_from(port).ok().filter(|port| *port != 0) {
            Some(port) => settings.port = port,
            None => errors.push(FieldError::new("port", "must be between 1 and 65535")),
        }
    }
    if let Some(url) = raw
        .public_url
        .as_deref()
        .map(str::trim)
        .filter(|url| !url.is_empty())
    {
        match Url::parse(url) {
            Ok(parsed)
                if matches!(parsed.scheme(), "http" | "https") && parsed.host_str().is_some() =>
            {
                settings.public_url = Some(url.to_string());
            }
            _ => errors.push(FieldError::new(
                "public_url",
                "must be an absolute http or https URL",
            )),
        }
    }
    for (index, proxy) in raw.trusted_proxies.iter().flatten().enumerate() {
        match parse_network(proxy) {
            Some(network) => settings.trusted_proxies.push(network),
            None => errors.push(FieldError::new(
                format!("trusted_proxies[{index}]"),
                "must be an IP address or a CIDR range such as 10.0.0.0/8",
            )),
        }
    }
    if errors.is_empty() {
        Ok(settings)
    } else {
        Err(errors)
    }
}

fn parse_network(text: &str) -> Option<IpNet> {
    let text = text.trim();
    text.parse::<IpNet>()
        .ok()
        .or_else(|| text.parse::<IpAddr>().ok().map(IpNet::from))
}
