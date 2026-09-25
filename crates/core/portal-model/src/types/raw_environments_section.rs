use std::collections::BTreeMap;

use serde::Deserialize;

use super::RawEnvironment;

#[derive(Debug, Default, Deserialize)]
pub struct RawEnvironmentsSection {
    #[serde(default)]
    pub environments: BTreeMap<String, RawEnvironment>,
}
