use std::sync::Arc;

use portal_config::ConfigStore;

use crate::ports::Connection;
use crate::services::{SessionStore, Throttle};

#[derive(Clone)]
pub struct AuthState {
    pub configuration: Arc<ConfigStore>,
    pub sessions: Arc<SessionStore>,
    pub throttle: Arc<Throttle>,
    pub connection: Arc<dyn Connection>,
}
