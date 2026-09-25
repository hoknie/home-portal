use rust_embed::RustEmbed;

use crate::ports::AssetSource;
use crate::types::Asset;

#[derive(RustEmbed)]
#[folder = "../../../web/out"]
#[allow_missing = true]
pub struct Embedded;

impl AssetSource for Embedded {
    fn get(&self, path: &str) -> Option<Asset> {
        <Embedded as RustEmbed>::get(path).map(|file| Asset {
            bytes: file.data,
            content_type: file.metadata.mimetype().to_string(),
        })
    }

    fn is_empty(&self) -> bool {
        <Embedded as RustEmbed>::iter().next().is_none()
    }
}
