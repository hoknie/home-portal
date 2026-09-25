use serde::{Deserialize, Serialize};

use crate::types::DownloadState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaddyResponse {
    pub managed: bool,
    pub installed: Option<String>,
    pub installed_from: Option<String>,
    pub source: String,
    pub version: String,
    pub release_url: String,
    pub platform: Option<String>,
    pub platform_error: Option<String>,
    pub download: DownloadState,
    pub log: Vec<String>,
}
