use std::sync::Arc;

use portal_config::{ConfigStore, Revisioned};
use portal_feature::ApiError;

use crate::loops::CaddySync;
use crate::types::ProxyView;

pub const DISABLED: &str = "the proxy is not enabled in the configuration";

#[derive(Clone)]
pub struct ApplyProxy {
    configuration: Arc<ConfigStore>,
    sync: Arc<CaddySync>,
}

impl ApplyProxy {
    pub fn new(configuration: Arc<ConfigStore>, sync: Arc<CaddySync>) -> ApplyProxy {
        ApplyProxy {
            configuration,
            sync,
        }
    }

    pub async fn run(&self) -> Result<Revisioned<ProxyView>, ApiError> {
        if !self.sync.settings().enabled {
            return Err(ApiError::Conflict(DISABLED.to_string()));
        }
        self.sync
            .apply()
            .await
            .map_err(|problem| ApiError::BadGateway(problem.to_string()))?;
        let revision = self.configuration.read().revision;
        Ok(Revisioned::new(self.sync.view(), revision))
    }
}
