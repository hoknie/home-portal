use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationLocation {
    pub path: PathBuf,
    pub by_default: bool,
}
