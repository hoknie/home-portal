use std::env;
use std::net::SocketAddr;

use portal_config::{ConfigError, ConfigStore};
use portal_network::{EffectiveAddress, read_network};

use crate::types::BootError;

pub const ADDRESS_VARIABLE: &str = "HOME_PORTAL_ADDRESS";

pub fn from_environment(store: &ConfigStore) -> Result<EffectiveAddress, BootError> {
    resolve_address(store, env::var(ADDRESS_VARIABLE).ok())
}

pub fn resolve_address(
    store: &ConfigStore,
    environment: Option<String>,
) -> Result<EffectiveAddress, BootError> {
    if let Some(value) = environment {
        return parse_address(value).map(|address| EffectiveAddress {
            address,
            overridden: true,
        });
    }
    let settings = read_network(&store.read().document).map_err(|errors| {
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
