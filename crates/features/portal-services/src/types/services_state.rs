use std::sync::Arc;

use portal_config::ConfigStore;

use crate::ports::Publishing;
use crate::services::{StatusBoard, Supervisor};

#[derive(Clone)]
pub struct ServicesState {
    pub configuration: Arc<ConfigStore>,
    pub board: Arc<StatusBoard>,
    pub supervisor: Arc<Supervisor>,
    pub publishing: Arc<dyn Publishing>,
}
