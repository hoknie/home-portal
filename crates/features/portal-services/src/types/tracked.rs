use portal_model::ServiceStatus;

use crate::services::ServiceHistory;

#[derive(Debug, Clone)]
pub struct Tracked {
    pub status: ServiceStatus,
    pub history: ServiceHistory,
}
