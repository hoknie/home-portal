use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FolderQuery {
    pub name: String,
}
