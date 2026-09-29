use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FolderRequest {
    #[serde(default)]
    pub name: Option<String>,
}
