use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use ipnet::IpNet;
use portal_model::{Environments, Language};
use portal_network::{NetworkOfDocument, NetworkSettings};
use portal_web::InterfaceOfDocument;
use toml_edit::DocumentMut;

use crate::boot::parse_address;
use crate::types::BootError;

#[derive(Debug, Clone)]
pub struct MainFile {
    pub address: SocketAddr,
    pub trusted: Vec<IpNet>,
    pub environments: Option<Environments>,
    pub language: Language,
}

impl MainFile {
    pub fn read(path: &Path) -> MainFile {
        fs::read_to_string(path)
            .ok()
            .and_then(|text| text.parse::<DocumentMut>().ok())
            .as_ref()
            .map_or_else(MainFile::defaults, MainFile::of)
    }

    pub fn listening(self, variable: Option<String>) -> Result<MainFile, BootError> {
        match variable {
            Some(value) => Ok(MainFile {
                address: parse_address(value)?,
                ..self
            }),
            None => Ok(self),
        }
    }

    pub fn defaults() -> MainFile {
        MainFile {
            address: NetworkSettings::default().socket_address(),
            trusted: Vec::new(),
            environments: None,
            language: Language::default(),
        }
    }

    pub fn of(document: &DocumentMut) -> MainFile {
        let reading = NetworkOfDocument.run(document);
        let settings = reading.settings.unwrap_or_default();
        MainFile {
            address: settings.socket_address(),
            trusted: settings.trusted_proxies,
            environments: reading.environments.ok(),
            language: InterfaceOfDocument.run(document).unwrap_or_default(),
        }
    }
}
