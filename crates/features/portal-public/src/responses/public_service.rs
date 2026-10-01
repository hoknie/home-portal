use portal_model::ServiceStatus;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PublicService {
    pub id: String,
    pub name: String,
    pub address: String,
    pub group: Option<String>,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub status: Option<ServiceStatus>,
}
