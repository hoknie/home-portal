use serde::Serialize;

use super::StepFieldResponse;
use crate::types::KindDescription;

#[derive(Debug, Clone, Serialize)]
pub struct StepKindResponse {
    pub name: String,
    pub group: String,
    pub fields: Vec<StepFieldResponse>,
    pub results: Vec<String>,
    pub exclusive: Vec<Vec<String>>,
}

impl StepKindResponse {
    pub fn of(kind: &KindDescription) -> StepKindResponse {
        StepKindResponse {
            name: kind.name.to_string(),
            group: kind.group.name().to_string(),
            fields: kind.fields.iter().map(StepFieldResponse::of).collect(),
            results: kind
                .results
                .iter()
                .map(|result| result.to_string())
                .collect(),
            exclusive: kind
                .exclusive
                .iter()
                .map(|group| group.iter().map(|field| field.to_string()).collect())
                .collect(),
        }
    }
}
