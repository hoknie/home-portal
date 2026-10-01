use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct TransformPreview {
    pub input: Result<Value, String>,
    pub steps: Vec<Result<Value, String>>,
    pub examples: Vec<Result<Value, String>>,
}
