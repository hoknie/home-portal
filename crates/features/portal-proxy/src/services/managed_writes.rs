use std::path::{Path, PathBuf};

use portal_config::{ConfigStore, Revision, Section};
use portal_feature::ApiError;

use crate::repositories::{SECTION, write_managed};

pub fn section_target(configuration: &ConfigStore) -> PathBuf {
    configuration
        .read()
        .origins
        .table(SECTION)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| configuration.home_of(Section::Proxy))
}

pub async fn store_managed(
    configuration: &ConfigStore,
    revision: &Revision,
    managed: bool,
) -> Result<(), ApiError> {
    let target = section_target(configuration);
    configuration
        .update(&target, revision, |document| {
            write_managed(document, managed);
            Ok(())
        })
        .await?;
    Ok(())
}
