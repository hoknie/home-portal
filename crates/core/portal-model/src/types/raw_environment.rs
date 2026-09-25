use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct RawEnvironment {
    #[serde(default)]
    pub networks: Vec<String>,
}
