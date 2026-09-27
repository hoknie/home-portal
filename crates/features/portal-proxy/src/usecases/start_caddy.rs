use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::ApiError;

use crate::clients::CaddyAdmin;
use crate::loops::CaddySync;
use crate::services::store_managed;
use crate::types::ProxyView;

pub const NOT_INSTALLED: &str = "Caddy is not downloaded yet";

#[derive(Clone)]
pub struct StartCaddy {
    configuration: Arc<ConfigStore>,
    sync: Arc<CaddySync>,
}

impl StartCaddy {
    pub fn new(configuration: Arc<ConfigStore>, sync: Arc<CaddySync>) -> StartCaddy {
        StartCaddy {
            configuration,
            sync,
        }
    }

    pub async fn run(
        &self,
        revision: Result<Revision, ApiError>,
    ) -> Result<Revisioned<ProxyView>, ApiError> {
        if self.sync.manager().installed().is_none() {
            return Err(ApiError::Conflict(NOT_INSTALLED.to_string()));
        }
        store_managed(&self.configuration, &revision?, true).await?;
        let settings = self.sync.settings();
        let admin = CaddyAdmin::new(&settings.admin).map_err(ApiError::Internal)?;
        if admin.config().await.is_err() && !CaddyAdmin::occupied(&settings.admin).await {
            self.sync
                .manager()
                .launch(&settings.admin)
                .map_err(ApiError::Internal)?;
        }
        let revision = self.configuration.read().revision;
        Ok(Revisioned::new(self.sync.view(), revision))
    }
}
