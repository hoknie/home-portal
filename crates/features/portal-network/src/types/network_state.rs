use std::sync::Arc;

use portal_config::ConfigStore;

use super::EffectiveAddress;

#[derive(Clone)]
pub struct NetworkState {
    pub configuration: Arc<ConfigStore>,
    pub effective: EffectiveAddress,
}
