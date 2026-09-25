use portal_model::ServiceStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicService {
    pub id: String,
    pub name: String,
    pub address: String,
    pub group: Option<String>,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub status: Option<ServiceStatus>,
}
