use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct RawCaddy {
    pub source: Option<String>,
    pub version: Option<String>,
}
