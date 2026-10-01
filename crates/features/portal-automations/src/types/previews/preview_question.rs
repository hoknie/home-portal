use std::collections::BTreeMap;

use serde_json::Value;

use crate::types::RawOperation;

#[derive(Debug, Clone, Default)]
pub struct PreviewQuestion {
    pub value: Value,
    pub filters: String,
    pub operations: Vec<RawOperation>,
    pub examples: Vec<String>,
    pub names: BTreeMap<String, Value>,
}
