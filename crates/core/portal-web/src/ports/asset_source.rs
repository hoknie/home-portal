use crate::types::Asset;

pub trait AssetSource {
    fn get(&self, path: &str) -> Option<Asset>;

    fn is_empty(&self) -> bool;
}
