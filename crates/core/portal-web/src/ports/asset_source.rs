use crate::types::Asset;

pub trait AssetSource: Send + Sync {
    fn get(&self, path: &str) -> Option<Asset>;

    fn is_empty(&self) -> bool;

    fn unavailable(&self) -> String;
}
