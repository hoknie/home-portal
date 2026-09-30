use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Group {
    pub name: String,
    #[serde(default)]
    pub permissions: BTreeMap<String, Vec<String>>,
}
