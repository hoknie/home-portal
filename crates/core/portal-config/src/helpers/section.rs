use serde::de::DeserializeOwned;
use toml_edit::DocumentMut;

pub fn deserialize_section<T: DeserializeOwned>(document: &DocumentMut) -> Result<T, String> {
    toml::from_str(&document.to_string()).map_err(|error| error.message().to_string())
}
