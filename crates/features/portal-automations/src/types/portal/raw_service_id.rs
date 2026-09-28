use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawServiceId {
    #[serde(default)]
    pub id: String,
}
