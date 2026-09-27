use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::ApiError;
use portal_model::Environment;

use crate::services::Supervisor;
use crate::types::{ServiceEntry, ServicesSection, Wake};

pub const PROBING_DISABLED: &str = "probing is disabled for this service";

#[derive(Clone)]
pub struct WakeProbe {
    configuration: Arc<ConfigStore>,
    supervisor: Arc<Supervisor>,
}

impl WakeProbe {
    pub fn new(configuration: Arc<ConfigStore>, supervisor: Arc<Supervisor>) -> WakeProbe {
        WakeProbe {
            configuration,
            supervisor,
        }
    }

    pub fn run(&self, id: &str, environment: &Environment) -> Result<(), ApiError> {
        let document = self.configuration.read().document;
        if ServicesSection::visible(&document, id, environment).is_none() {
            return Err(ApiError::NotFound(ServiceEntry::UNKNOWN));
        }
        match self.supervisor.wake(id) {
            Wake::Woken => Ok(()),
            Wake::NotFound => Err(ApiError::NotFound(ServiceEntry::UNKNOWN)),
            Wake::Disabled => Err(ApiError::Conflict(PROBING_DISABLED.to_string())),
            Wake::TooSoon(wait) => Err(ApiError::TooManyRequests {
                retry_after_seconds: wait.as_secs().max(1),
            }),
        }
    }
}
