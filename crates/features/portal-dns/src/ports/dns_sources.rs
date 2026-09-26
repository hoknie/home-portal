use std::net::IpAddr;

use portal_model::Environments;

use crate::types::PublishedHost;

pub trait DnsSources: Send + Sync {
    fn environments(&self) -> Environments;

    fn published(&self) -> Vec<PublishedHost>;

    fn interfaces(&self) -> Vec<IpAddr>;
}
