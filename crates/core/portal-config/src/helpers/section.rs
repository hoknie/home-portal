use serde::de::DeserializeOwned;
use toml_edit::DocumentMut;

pub fn deserialize_section<T: DeserializeOwned>(document: &DocumentMut) -> Result<T, String> {
    toml_edit::de::from_document(document.clone()).map_err(|error| error.message().to_string())
}
