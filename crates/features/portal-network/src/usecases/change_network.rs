use std::path::Path;
use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::ApiError;

use crate::repositories::{SECTION, write_network};
use crate::services::{check_network, configured_network};
use crate::types::{NetworkSettings, RawNetwork};

#[derive(Clone)]
pub struct ChangeNetwork {
    configuration: Arc<ConfigStore>,
}

impl ChangeNetwork {
    pub fn new(configuration: Arc<ConfigStore>) -> ChangeNetwork {
        ChangeNetwork { configuration }
    }

    pub async fn run(
        &self,
        raw: &RawNetwork,
        revision: &Revision,
    ) -> Result<Revisioned<NetworkSettings>, ApiError> {
        let settings = check_network(raw).map_err(ApiError::Invalid)?;
        let target = self
            .configuration
            .read()
            .origins
            .table(SECTION)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.configuration.writes_to());
        let (_, snapshot) = self
            .configuration
            .update(&target, revision, |document| {
                write_network(document, &settings);
                Ok(())
            })
            .await?;
        let configured = configured_network(&snapshot.document)?;
        Ok(Revisioned::new(configured, snapshot.revision))
    }
}
