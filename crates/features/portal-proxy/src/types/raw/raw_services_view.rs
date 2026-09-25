use serde::Deserialize;

use super::RawPublishedService;

#[derive(Debug, Default, Deserialize)]
pub struct RawServicesView {
    #[serde(default)]
    pub services: Vec<RawPublishedService>,
}
