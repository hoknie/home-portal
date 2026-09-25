use std::sync::Arc;

use portal_config::ConfigStore;
use portal_network::EffectiveAddress;

pub struct Wiring {
    pub configuration: Arc<ConfigStore>,
    pub effective: EffectiveAddress,
}
