use std::sync::Arc;

use portal_config::ConfigStore;

use crate::loops::DnsRuntime;

#[derive(Clone)]
pub struct DnsContext {
    pub configuration: Arc<ConfigStore>,
    pub runtime: Arc<DnsRuntime>,
}
