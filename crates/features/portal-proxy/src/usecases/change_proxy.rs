use std::path::Path;
use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::ApiError;

use crate::loops::CaddySync;
use crate::repositories::{NETWORK, SECTION, trust_loopback, write_choice};
use crate::types::{ProxyChoice, ProxyView};

#[derive(Clone)]
pub struct ChangeProxy {
    configuration: Arc<ConfigStore>,
    sync: Arc<CaddySync>,
}

impl ChangeProxy {
    pub fn new(configuration: Arc<ConfigStore>, sync: Arc<CaddySync>) -> ChangeProxy {
        ChangeProxy {
            configuration,
            sync,
        }
    }

    pub async fn run(
        &self,
        choice: &ProxyChoice,
        revision: &Revision,
    ) -> Result<Revisioned<ProxyView>, ApiError> {
        let snapshot = self.configuration.read();
        let target = snapshot
            .origins
            .table(SECTION)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.configuration.writes_to());
        let network_here = snapshot
            .origins
            .table(NETWORK)
            .is_none_or(|origin| origin == target);
        self.configuration
            .update(&target, revision, |document| {
                write_choice(document, choice);
                if choice.enabled && network_here {
                    trust_loopback(document);
                }
                Ok(())
            })
            .await?;
        let revision = self.configuration.read().revision;
        Ok(Revisioned::new(self.sync.view(), revision))
    }
}
