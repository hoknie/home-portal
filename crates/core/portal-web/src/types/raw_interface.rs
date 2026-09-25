use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct RawInterfaceSection {
    #[serde(default)]
    pub interface: RawInterface,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawInterface {
    pub default_language: Option<String>,
}
