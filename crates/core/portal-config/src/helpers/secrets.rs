use std::collections::BTreeMap;

use toml_edit::{DocumentMut, Item};

use crate::types::SecretString;

pub const SECRETS_KEY: &str = "secrets";

pub fn take_secrets(document: &mut DocumentMut) -> BTreeMap<String, SecretString> {
    let mut secrets = BTreeMap::new();
    if let Some(table) = document.get(SECRETS_KEY).and_then(Item::as_table) {
        for (name, value) in table.iter() {
            if let Some(text) = value.as_str() {
                secrets.insert(name.to_string(), SecretString::new(text));
            }
        }
    }
    document.remove(SECRETS_KEY);
    secrets
}

pub fn holds_secrets(document: &DocumentMut) -> bool {
    document.get(SECRETS_KEY).is_some()
}
