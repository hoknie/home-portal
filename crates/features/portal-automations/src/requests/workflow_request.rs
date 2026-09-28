use portal_feature::FieldError;
use serde::Deserialize;
use serde_json::Value;

use crate::types::{RawInput, RawStep, RawWorkflow};

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowRequest {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub timeout_seconds: Option<i64>,
    #[serde(default)]
    pub inputs: Vec<RawInput>,
    #[serde(default)]
    pub steps: Vec<Value>,
}

impl WorkflowRequest {
    pub const NOT_A_STEP: &'static str = "is not a step";

    pub fn into_raw(self) -> Result<RawWorkflow, Vec<FieldError>> {
        let mut errors = Vec::new();
        let steps: Vec<RawStep> = self
            .steps
            .into_iter()
            .enumerate()
            .filter_map(|(index, step)| {
                serde_json::from_value(step)
                    .map_err(|error| {
                        errors.push(FieldError::new(
                            format!("steps[{index}]"),
                            format!("{}: {error}", Self::NOT_A_STEP),
                        ))
                    })
                    .ok()
            })
            .collect();
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(RawWorkflow {
            id: self.id,
            title: self.title,
            enabled: self.enabled,
            description: self.description.filter(|text| !text.is_empty()),
            tags: self.tags,
            timeout_seconds: self.timeout_seconds,
            inputs: self.inputs,
            steps,
        })
    }
}
