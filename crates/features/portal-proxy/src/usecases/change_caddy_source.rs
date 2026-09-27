use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::ApiError;

use crate::loops::CaddySync;
use crate::repositories::write_caddy_source;
use crate::services::section_target;
use crate::types::{CaddySource, ProxyView};

#[derive(Clone)]
pub struct ChangeCaddySource {
    configuration: Arc<ConfigStore>,
    sync: Arc<CaddySync>,
}

impl ChangeCaddySource {
    pub fn new(configuration: Arc<ConfigStore>, sync: Arc<CaddySync>) -> ChangeCaddySource {
        ChangeCaddySource {
            configuration,
            sync,
        }
    }

    pub async fn run(
        &self,
        source: &CaddySource,
        revision: &Revision,
    ) -> Result<Revisioned<ProxyView>, ApiError> {
        let target = section_target(&self.configuration);
        self.configuration
            .update(&target, revision, |document| {
                write_caddy_source(document, source);
                Ok(())
            })
            .await?;
        let revision = self.configuration.read().revision;
        Ok(Revisioned::new(self.sync.view(), revision))
    }
}
