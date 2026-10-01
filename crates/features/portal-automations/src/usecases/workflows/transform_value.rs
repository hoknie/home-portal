use crate::services::preview_transform;
use crate::types::{PreviewQuestion, TransformPreview};

#[derive(Clone, Default)]
pub struct TransformValue;

impl TransformValue {
    pub fn run(&self, question: &PreviewQuestion) -> Result<TransformPreview, String> {
        preview_transform(question)
    }
}
