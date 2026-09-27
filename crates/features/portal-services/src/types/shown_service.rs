use portal_model::{Environment, ServiceStatus};

use super::ServiceEntry;

#[derive(Debug, Clone)]
pub struct ShownService {
    pub entry: ServiceEntry,
    pub status: ServiceStatus,
    pub host: Environment,
    pub publishing: Option<u16>,
}
