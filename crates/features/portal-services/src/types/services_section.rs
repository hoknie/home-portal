use portal_config::deserialize_section;
use portal_model::Environment;
use serde::Deserialize;
use toml_edit::DocumentMut;

use super::ServiceEntry;

#[derive(Debug, Default, Deserialize)]
pub struct ServicesSection {
    #[serde(default)]
    pub services: Vec<ServiceEntry>,
}

impl ServicesSection {
    pub fn read(document: &DocumentMut) -> Result<ServicesSection, String> {
        deserialize_section(document)
    }

    pub fn visible(
        document: &DocumentMut,
        id: &str,
        environment: &Environment,
    ) -> Option<ServiceEntry> {
        ServicesSection::read(document)
            .ok()?
            .services
            .into_iter()
            .find(|entry| entry.id == id && entry.visible_to(environment))
    }
}
