use std::net::{Ipv4Addr, Ipv6Addr};

use super::RecordKind;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RecordData {
    A(Ipv4Addr),
    Aaaa(Ipv6Addr),
    Cname(String),
    Txt(String),
    Soa {
        primary: String,
        mailbox: String,
        serial: u32,
        minimum: u32,
    },
    Ns(String),
}

impl RecordData {
    pub fn kind(&self) -> RecordKind {
        match self {
            RecordData::A(_) => RecordKind::A,
            RecordData::Aaaa(_) => RecordKind::Aaaa,
            RecordData::Cname(_) => RecordKind::Cname,
            RecordData::Txt(_) => RecordKind::Txt,
            RecordData::Soa { .. } => RecordKind::Soa,
            RecordData::Ns(_) => RecordKind::Ns,
        }
    }

    pub fn text(&self) -> String {
        match self {
            RecordData::A(address) => address.to_string(),
            RecordData::Aaaa(address) => address.to_string(),
            RecordData::Cname(target) | RecordData::Ns(target) => target.clone(),
            RecordData::Txt(text) => text.clone(),
            RecordData::Soa {
                primary, serial, ..
            } => format!("{primary} {serial}"),
        }
    }
}
