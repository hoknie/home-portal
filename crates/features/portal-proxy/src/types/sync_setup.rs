use std::net::SocketAddr;
use std::sync::Arc;

use super::Cadence;
use crate::services::CaddyManager;

#[derive(Clone)]
pub struct SyncSetup {
    pub portal: SocketAddr,
    pub cadence: Cadence,
    pub manager: Arc<CaddyManager>,
}
