use serde::{Deserialize, Serialize};

use super::InterfaceResponse;
use crate::types::{EffectiveAddress, NetworkSettings};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkResponse {
    pub configured: NetworkSettings,
    pub effective: EffectiveAddress,
    pub restart_required: bool,
    pub interfaces: Vec<InterfaceResponse>,
}

impl NetworkResponse {
    pub fn of(
        configured: NetworkSettings,
        effective: EffectiveAddress,
        interfaces: Vec<InterfaceResponse>,
    ) -> NetworkResponse {
        let restart_required =
            !effective.overridden && configured.socket_address() != effective.address;
        NetworkResponse {
            configured,
            effective,
            restart_required,
            interfaces,
        }
    }
}
