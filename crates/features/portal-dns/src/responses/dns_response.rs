use serde::Serialize;

use super::{
    DnsSettingsResponse, EnvironmentAnswerResponse, NameResponse, RecordResponse,
    TransportResponse, ZoneResponse,
};
use crate::types::{DnsSettings, DnsState, TransportState, ZoneBook};

pub const HTTPS_OFF: &str = "DNS over HTTPS is off";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DnsResponse {
    pub enabled: bool,
    pub plain: TransportResponse,
    pub tls: TransportResponse,
    pub https: TransportResponse,
    pub last_error: Option<String>,
    pub zones: Vec<ZoneResponse>,
    pub names: Vec<NameResponse>,
    pub environments: Vec<String>,
    pub unaddressed: Vec<String>,
    pub tls_host: Option<String>,
    pub doh_url: Option<String>,
    pub settings: DnsSettingsResponse,
}

impl DnsResponse {
    pub fn of(settings: &DnsSettings, book: &ZoneBook, state: &DnsState) -> DnsResponse {
        let https = if settings.enabled && settings.https.enabled {
            TransportState {
                listening: true,
                address: None,
                reason: None,
            }
        } else {
            TransportState::off(HTTPS_OFF)
        };
        DnsResponse {
            enabled: settings.enabled,
            plain: TransportResponse::of(&state.plain),
            tls: TransportResponse::of(&state.tls),
            https: TransportResponse::of(&https),
            last_error: state.last_error.clone(),
            zones: book
                .zones
                .iter()
                .map(|zone| ZoneResponse {
                    apex: zone.apex.clone(),
                    single: zone.single,
                    serial: book.serial,
                })
                .collect(),
            names: book
                .names
                .iter()
                .map(|(name, environments)| NameResponse {
                    name: name.clone(),
                    answers: environments
                        .iter()
                        .map(|(environment, records)| EnvironmentAnswerResponse {
                            environment: environment.as_str().to_string(),
                            records: records
                                .iter()
                                .map(|data| RecordResponse {
                                    kind: data.kind().name().to_string(),
                                    value: data.text(),
                                })
                                .collect(),
                        })
                        .collect(),
                })
                .collect(),
            environments: book
                .environments
                .named()
                .iter()
                .map(|(environment, _)| environment.as_str().to_string())
                .collect(),
            unaddressed: book
                .unaddressed
                .iter()
                .map(|environment| environment.as_str().to_string())
                .collect(),
            tls_host: settings
                .tls
                .enabled
                .then(|| settings.secure_host().map(str::to_string))
                .flatten(),
            doh_url: settings.https.enabled.then(|| settings.doh_url()).flatten(),
            settings: DnsSettingsResponse::of(settings),
        }
    }
}
