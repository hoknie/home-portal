use portal_config::deserialize_section;
use portal_feature::FieldError;
use serde::Deserialize;
use toml_edit::DocumentMut;

use crate::types::IconSource;

#[derive(Debug, Default, Deserialize)]
struct IconsSection {
    #[serde(default)]
    services: Vec<IconEntry>,
}

#[derive(Debug, Deserialize)]
struct IconEntry {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    icon: Option<String>,
}

pub fn validate_icons(document: &DocumentMut) -> Vec<FieldError> {
    let section: IconsSection = match deserialize_section(document) {
        Ok(section) => section,
        Err(_) => return Vec::new(),
    };
    section
        .services
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let icon = entry.icon.as_deref()?;
            let problem = IconSource::parse(icon).err()?;
            let name = entry.id.clone().unwrap_or_else(|| index.to_string());
            Some(FieldError::new(
                format!("services[{index}].icon"),
                format!("{name}: {problem}"),
            ))
        })
        .collect()
}
