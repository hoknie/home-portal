use std::net::SocketAddr;
use std::path::PathBuf;

use axum::http::{HeaderMap, HeaderValue};
use portal_config::ConfigError;
use portal_feature::FieldError;

use crate::failures::main_file::MainFile;
use crate::types::BootError;

pub const HOST: &str = "127.0.0.1:50000";
pub const PHONE: &str = "192.168.1.40:50000";
pub const OUTSIDE: &str = "203.0.113.9:50000";
pub const LOCAL: &str = "[environments.local]\nnetworks = [\"192.168.1.0/24\"]\n";

pub fn main_of(text: &str) -> MainFile {
    MainFile::of(&text.parse().unwrap())
}

pub fn peer(address: &str) -> Option<SocketAddr> {
    Some(address.parse().unwrap())
}

pub fn forwarded_for(address: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-forwarded-for", HeaderValue::from_str(address).unwrap());
    headers
}

pub fn invalid(fields: &[&str]) -> BootError {
    BootError::Configuration(ConfigError::Invalid {
        path: PathBuf::from("/home/ad/.config/home-portal/home-portal.toml"),
        errors: fields
            .iter()
            .map(|field| FieldError::new(*field, "is no longer read"))
            .collect(),
    })
}
