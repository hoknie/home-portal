use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::ApiError;

use crate::clients::CaddyAdmin;
use crate::loops::CaddySync;
use crate::services::store_managed;
use crate::types::ProxyView;

#[derive(Clone)]
pub struct StopCaddy {
    configuration: Arc<ConfigStore>,
    sync: Arc<CaddySync>,
}

impl StopCaddy {
    pub fn new(configuration: Arc<ConfigStore>, sync: Arc<CaddySync>) -> StopCaddy {
        StopCaddy {
            configuration,
            sync,
        }
    }

    pub async fn run(&self, revision: &Revision) -> Result<Revisioned<ProxyView>, ApiError> {
        store_managed(&self.configuration, revision, false).await?;
        let settings = self.sync.settings();
        let admin = CaddyAdmin::new(&settings.admin).map_err(ApiError::Internal)?;
        if let Err(problem) = admin.stop().await {
            tracing::info!(%problem, "caddy was not running");
        }
        let revision = self.configuration.read().revision;
        Ok(Revisioned::new(self.sync.view(), revision))
    }
}
