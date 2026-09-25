use std::sync::Arc;

use portal_config::ConfigStore;

use super::ProxyPorts;
use crate::loops::CaddySync;

#[derive(Clone)]
pub struct ProxyState {
    pub configuration: Arc<ConfigStore>,
    pub ports: ProxyPorts,
    pub sync: Arc<CaddySync>,
}
