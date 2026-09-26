use std::collections::BTreeMap;

use portal_model::{RawEnvironment, TlsPolicy};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct RawDnsSection {
    #[serde(default)]
    pub dns: RawDns,
    #[serde(default)]
    pub proxy: RawProxyView,
    #[serde(default)]
    pub environments: BTreeMap<String, RawEnvironment>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawDns {
    pub enabled: Option<bool>,
    pub address: Option<String>,
    pub port: Option<i64>,
    #[serde(default)]
    pub zones: Vec<String>,
    pub ttl: Option<i64>,
    #[serde(default)]
    pub addresses: BTreeMap<String, RawAddresses>,
    #[serde(default)]
    pub records: Vec<RawRecord>,
    #[serde(default)]
    pub tls: RawDnsTls,
    #[serde(default)]
    pub https: RawDnsHttps,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RawAddresses {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Default, Deserialize)]
pub struct RawRecord {
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub value: String,
    pub environments: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawDnsTls {
    pub enabled: Option<bool>,
    pub port: Option<i64>,
    pub certificate: Option<String>,
    pub key: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawDnsHttps {
    pub enabled: Option<bool>,
    pub host: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawProxyView {
    pub enabled: Option<bool>,
    pub managed: Option<bool>,
    pub portal_host: Option<String>,
    pub https_port: Option<i64>,
    pub tls: Option<TlsPolicy>,
}
