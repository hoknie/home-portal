use portal_feature::FieldError;
use serde::Deserialize;

use crate::types::{CaddySource, CaddyVersion};

#[derive(Debug, Deserialize)]
pub struct CaddySourceRequest {
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

impl CaddySourceRequest {
    pub const SOURCE_FIELD: &'static str = "proxy.caddy.source";
    pub const VERSION_FIELD: &'static str = "proxy.caddy.version";

    pub fn into_source(self) -> Result<CaddySource, Vec<FieldError>> {
        let given = |text: Option<String>| text.filter(|text| !text.trim().is_empty());
        let mut source = CaddySource::default();
        let mut errors = Vec::new();
        if let Some(base) = given(self.source) {
            match CaddySource::parse_base(&base) {
                Ok(base) => source.base = base,
                Err(message) => errors.push(FieldError::new(Self::SOURCE_FIELD, message)),
            }
        }
        if let Some(version) = given(self.version) {
            match CaddyVersion::parse(&version) {
                Ok(version) => source.version = version,
                Err(message) => errors.push(FieldError::new(Self::VERSION_FIELD, message)),
            }
        }
        if errors.is_empty() {
            Ok(source)
        } else {
            Err(errors)
        }
    }
}
