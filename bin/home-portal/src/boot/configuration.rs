use std::sync::Arc;

use portal_config::{ConfigStore, configuration_path};

use crate::types::{BootError, Registry};

pub fn open() -> Result<Arc<ConfigStore>, BootError> {
    let store = ConfigStore::open(configuration_path())?;
    tracing::info!(path = %store.path().display(), "configuration loaded");
    Ok(Arc::new(store))
}

pub fn adopt(store: &ConfigStore, registry: &Registry) -> Result<(), BootError> {
    let mut validators: Vec<_> = registry
        .features
        .iter()
        .filter_map(|feature| feature.validator())
        .collect();
    validators.push(portal_web::validate_interface);
    store.adopt(validators)?;
    store.adopt_checks(vec![registry.widgets.checker()])?;
    Ok(())
}
