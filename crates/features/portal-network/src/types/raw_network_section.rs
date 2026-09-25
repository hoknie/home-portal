use serde::Deserialize;

use super::RawNetwork;

#[derive(Debug, Default, Deserialize)]
pub struct RawNetworkSection {
    #[serde(default)]
    pub network: RawNetwork,
}
