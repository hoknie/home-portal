use serde_json::Value;

use crate::services::transform_sample;
use crate::types::RawOperation;

#[derive(Clone, Default)]
pub struct TransformValue;

impl TransformValue {
    pub fn run(&self, input: Value, operations: &[RawOperation]) -> Result<Value, String> {
        transform_sample(input, operations)
    }
}
