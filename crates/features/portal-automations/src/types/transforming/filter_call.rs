use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterCall {
    pub name: String,
    pub arguments: Vec<Value>,
}
