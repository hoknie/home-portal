use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaddyHome {
    pub directory: PathBuf,
}

impl CaddyHome {
    pub const DIRECTORY: &'static str = "caddy";
    pub const BINARY: &'static str = "caddy";
    pub const VERSION_FILE: &'static str = "VERSION";
    pub const ORIGIN_FILE: &'static str = "ORIGIN";
    pub const LOG: &'static str = "caddy.log";
    pub const INITIAL: &'static str = "initial.json";
    pub const DATA: &'static str = "data";
    pub const CONFIG: &'static str = "config";
    pub const STAGING: &'static str = "staging";

    pub fn beside(configuration: &Path) -> CaddyHome {
        let parent = configuration
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let directory = parent.join(Self::DIRECTORY);
        CaddyHome {
            directory: std::path::absolute(&directory).unwrap_or(directory),
        }
    }

    pub fn binary(&self) -> PathBuf {
        self.directory.join(Self::BINARY)
    }

    pub fn log(&self) -> PathBuf {
        self.directory.join(Self::LOG)
    }

    pub fn initial(&self) -> PathBuf {
        self.directory.join(Self::INITIAL)
    }

    pub fn data(&self) -> PathBuf {
        self.directory.join(Self::DATA)
    }

    pub fn config(&self) -> PathBuf {
        self.directory.join(Self::CONFIG)
    }

    pub fn staging(&self) -> PathBuf {
        self.directory.join(Self::STAGING)
    }

    pub fn version_file(&self) -> PathBuf {
        self.directory.join(Self::VERSION_FILE)
    }

    pub fn origin_file(&self) -> PathBuf {
        self.directory.join(Self::ORIGIN_FILE)
    }

    pub fn installed_from(&self) -> Option<String> {
        if !self.binary().is_file() {
            return None;
        }
        let origin = fs::read_to_string(self.origin_file()).ok()?;
        Some(origin.trim().to_string()).filter(|origin| !origin.is_empty())
    }

    pub fn installed(&self) -> Option<String> {
        if !self.binary().is_file() {
            return None;
        }
        let version = fs::read_to_string(self.version_file()).unwrap_or_default();
        Some(version.trim().to_string())
    }
}
