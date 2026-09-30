use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct GroupWriteRequest {
    pub name: String,
    #[serde(default)]
    pub rights: BTreeMap<String, Vec<String>>,
}
