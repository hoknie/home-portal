use std::collections::BTreeMap;

use serde::Serialize;

use crate::types::DnsSettings;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DnsSettingsResponse {
    pub address: String,
    pub port: u16,
    pub zones: Vec<String>,
    pub ttl: u32,
    pub addresses: BTreeMap<String, Vec<String>>,
    pub tls: DnsTlsSettingsResponse,
    pub https: DnsHttpsSettingsResponse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DnsTlsSettingsResponse {
    pub enabled: bool,
    pub port: u16,
    pub certificate: Option<String>,
    pub key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DnsHttpsSettingsResponse {
    pub enabled: bool,
    pub host: Option<String>,
}

impl DnsSettingsResponse {
    pub fn of(settings: &DnsSettings) -> DnsSettingsResponse {
        DnsSettingsResponse {
            address: settings.address.to_string(),
            port: settings.port,
            zones: settings.zones.clone(),
            ttl: settings.ttl,
            addresses: settings
                .addresses
                .iter()
                .map(|(environment, found)| {
                    let texts = found
                        .v4
                        .map(|address| address.to_string())
                        .into_iter()
                        .chain(found.v6.map(|address| address.to_string()))
                        .collect();
                    (environment.as_str().to_string(), texts)
                })
                .collect(),
            tls: DnsTlsSettingsResponse {
                enabled: settings.tls.enabled,
                port: settings.tls.port,
                certificate: settings
                    .tls
                    .certificate
                    .as_ref()
                    .map(|path| path.display().to_string()),
                key: settings
                    .tls
                    .key
                    .as_ref()
                    .map(|path| path.display().to_string()),
            },
            https: DnsHttpsSettingsResponse {
                enabled: settings.https.enabled,
                host: settings.https.host.clone(),
            },
        }
    }
}
