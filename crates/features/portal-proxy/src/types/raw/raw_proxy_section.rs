use serde::Deserialize;

use super::{RawNetworkView, RawProxy};

#[derive(Debug, Default, Deserialize)]
pub struct RawProxySection {
    #[serde(default)]
    pub proxy: RawProxy,
    #[serde(default)]
    pub network: RawNetworkView,
}
