use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use portal_config::{ConfigError, ConfigStore};
use portal_network::{CurrentNetwork, EffectiveAddress};

use crate::types::BootError;

pub const ADDRESS_VARIABLE: &str = "HOME_PORTAL_ADDRESS";

pub fn from_environment(store: &Arc<ConfigStore>) -> Result<EffectiveAddress, BootError> {
    resolve_address(store, env::var(ADDRESS_VARIABLE).ok())
}

pub fn resolve_address(
    store: &Arc<ConfigStore>,
    environment: Option<String>,
) -> Result<EffectiveAddress, BootError> {
    if let Some(value) = environment {
        return parse_address(value).map(|address| EffectiveAddress {
            address,
            overridden: true,
        });
    }
    let settings = CurrentNetwork::new(store.clone())
        .run()
        .settings
        .map_err(|errors| {
            BootError::Configuration(ConfigError::Invalid {
                path: store.path().to_path_buf(),
                errors,
            })
        })?;
    Ok(EffectiveAddress {
        address: settings.socket_address(),
        overridden: false,
    })
}

pub fn parse_address(value: String) -> Result<SocketAddr, BootError> {
    value.parse().map_err(|_| BootError::Address {
        value,
        variable: ADDRESS_VARIABLE,
    })
}
