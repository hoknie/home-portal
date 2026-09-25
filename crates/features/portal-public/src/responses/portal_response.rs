use portal_model::Environment;
use serde::{Deserialize, Serialize};

use super::{PublicSection, PublicService, PublicWidget};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortalResponse {
    pub environment: Environment,
    pub detected: Environment,
    pub switchable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environments: Option<Vec<Environment>>,
    pub sections: Vec<PublicSection>,
    pub services: Vec<PublicService>,
    pub widgets: Vec<PublicWidget>,
}
