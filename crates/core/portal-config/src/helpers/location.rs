use std::env;
use std::path::PathBuf;

pub const CONFIGURATION_VARIABLE: &str = "HOME_PORTAL_CONFIG";
pub const DEFAULT_CONFIGURATION_FILE: &str = "home-portal.toml";

pub fn configuration_path() -> PathBuf {
    resolve_configuration_path(env::var(CONFIGURATION_VARIABLE).ok())
}

pub fn resolve_configuration_path(value: Option<String>) -> PathBuf {
    PathBuf::from(value.unwrap_or_else(|| DEFAULT_CONFIGURATION_FILE.to_string()))
}
