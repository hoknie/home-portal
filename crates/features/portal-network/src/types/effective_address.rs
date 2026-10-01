use std::net::SocketAddr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EffectiveAddress {
    pub address: SocketAddr,
    pub overridden: bool,
}
