use std::sync::Arc;

use portal_config::{ConfigStore, Revisioned};
use portal_feature::ApiError;

use crate::loops::CaddySync;
use crate::types::ProxyView;

#[derive(Clone)]
pub struct DownloadCaddy {
    configuration: Arc<ConfigStore>,
    sync: Arc<CaddySync>,
}

impl DownloadCaddy {
    pub fn new(configuration: Arc<ConfigStore>, sync: Arc<CaddySync>) -> DownloadCaddy {
        DownloadCaddy {
            configuration,
            sync,
        }
    }

    pub fn run(&self) -> Result<Revisioned<ProxyView>, ApiError> {
        let source = self.sync.settings().caddy;
        self.sync
            .manager()
            .begin_download(source)
            .map_err(|message| ApiError::Conflict(message.to_string()))?;
        let revision = self.configuration.read().revision;
        Ok(Revisioned::new(self.sync.view(), revision))
    }
}
