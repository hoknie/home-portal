use portal_feature::FieldError;
use portal_model::Environments;

use super::NetworkSettings;

#[derive(Debug, Clone)]
pub struct NetworkReading {
    pub settings: Result<NetworkSettings, Vec<FieldError>>,
    pub environments: Result<Environments, Vec<FieldError>>,
}
