use std::sync::Arc;

use portal_config::ConfigStore;
use portal_proxy::read_settings;
use portal_services::Publishing;

pub struct ProxyPublishing {
    pub configuration: Arc<ConfigStore>,
}

impl Publishing for ProxyPublishing {
    fn https_port(&self) -> Option<u16> {
        read_settings(&self.configuration.read().document)
            .ok()
            .filter(|settings| settings.enabled)
            .map(|settings| settings.https_port)
    }
}
