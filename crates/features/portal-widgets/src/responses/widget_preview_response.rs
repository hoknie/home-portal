use portal_feature::FieldError;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;

use super::RenderedBlock;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct PreviewErrorResponse {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct WidgetPreviewResponse {
    pub blocks: Vec<RenderedBlock>,
    pub errors: Vec<PreviewErrorResponse>,
    pub data: Value,
    pub ran: bool,
    pub problem: Option<String>,
    pub paths: Vec<DeclaredPathResponse>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DeclaredPathResponse {
    pub path: String,
    pub description: Option<String>,
    pub kind: String,
}

impl DeclaredPathResponse {
    pub fn of(declared: crate::ports::DeclaredPath) -> DeclaredPathResponse {
        DeclaredPathResponse {
            path: declared.path,
            description: declared.description,
            kind: declared.kind,
        }
    }
}

impl PreviewErrorResponse {
    pub fn of(error: FieldError) -> PreviewErrorResponse {
        PreviewErrorResponse {
            field: error.field,
            message: error.message,
        }
    }
}
