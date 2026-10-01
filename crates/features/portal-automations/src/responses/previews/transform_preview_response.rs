use schemars::JsonSchema;
use serde::Serialize;

use super::PreviewStepResponse;
use crate::types::TransformPreview;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TransformPreviewResponse {
    pub input: PreviewStepResponse,
    pub steps: Vec<PreviewStepResponse>,
    pub examples: Vec<PreviewStepResponse>,
}

impl TransformPreviewResponse {
    pub fn of(preview: &TransformPreview) -> TransformPreviewResponse {
        TransformPreviewResponse {
            input: PreviewStepResponse::of(&preview.input),
            steps: preview.steps.iter().map(PreviewStepResponse::of).collect(),
            examples: preview
                .examples
                .iter()
                .map(PreviewStepResponse::of)
                .collect(),
        }
    }
}
