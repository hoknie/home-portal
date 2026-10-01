use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use crate::types::{PreviewQuestion, RawOperation};

#[derive(Debug, Clone, Deserialize)]
pub struct TransformPreviewRequest {
    pub value: Value,
    #[serde(default)]
    pub filters: String,
    #[serde(default)]
    pub operations: Vec<RawOperation>,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default)]
    pub names: BTreeMap<String, Value>,
}

impl TransformPreviewRequest {
    pub fn question(self) -> PreviewQuestion {
        PreviewQuestion {
            value: self.value,
            filters: self.filters,
            operations: self.operations,
            examples: self.examples,
            names: self.names,
        }
    }
}
