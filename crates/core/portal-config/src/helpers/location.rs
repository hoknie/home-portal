use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::types::{ConfigError, ConfigurationLocation};

pub const CONFIGURATION_VARIABLE: &str = "HOME_PORTAL_CONFIG";
pub const XDG_CONFIGURATION_VARIABLE: &str = "XDG_CONFIG_HOME";
pub const HOME_VARIABLE: &str = "HOME";
pub const CONFIGURATION_FOLDER: &str = "home-portal";
pub const CONFIGURATION_FILE: &str = "home-portal.toml";
pub const HOME_CONFIGURATION_FOLDER: &str = ".config";

pub fn configuration_path() -> Result<ConfigurationLocation, ConfigError> {
    resolve_configuration_path(
        env::var_os(CONFIGURATION_VARIABLE),
        env::var_os(XDG_CONFIGURATION_VARIABLE),
        env::var_os(HOME_VARIABLE),
    )
}

pub fn resolve_configuration_path(
    configuration: Option<OsString>,
    xdg: Option<OsString>,
    home: Option<OsString>,
) -> Result<ConfigurationLocation, ConfigError> {
    let present =
        |value: Option<OsString>| value.filter(|value| !value.is_empty()).map(PathBuf::from);
    if let Some(path) = present(configuration) {
        return Ok(ConfigurationLocation {
            path,
            by_default: false,
        });
    }
    let base = present(xdg)
        .filter(|path| path.is_absolute())
        .or_else(|| present(home).map(|home| home.join(HOME_CONFIGURATION_FOLDER)))
        .ok_or(ConfigError::NoConfigurationPath)?;
    Ok(ConfigurationLocation {
        path: base.join(CONFIGURATION_FOLDER).join(CONFIGURATION_FILE),
        by_default: true,
    })
}

pub fn stray_configuration(working: &Path) -> Option<PathBuf> {
    let stray = working.join(CONFIGURATION_FILE);
    stray.is_file().then_some(stray)
}
