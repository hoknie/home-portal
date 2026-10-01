use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-open" = "idle"))]
#[serde(rename_all = "lowercase")]
pub enum DownloadStage {
    #[default]
    Idle,
    Downloading,
    Installed,
    Failed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DownloadState {
    pub state: DownloadStage,
    pub error: Option<String>,
}
