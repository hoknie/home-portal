use portal_feature::ApiError;
use toml_edit::DocumentMut;

use crate::repositories::workflow_position;
use crate::services::entry_errors;
use crate::types::{AutomationsSection, Workflow};

pub fn checked(document: &DocumentMut, id: &str) -> Result<(), ApiError> {
    let index = workflow_position(document, id).ok_or(ApiError::NotFound(Workflow::UNKNOWN))?;
    let section = AutomationsSection::read(document).map_err(ApiError::Internal)?;
    let errors = entry_errors(&section, index);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ApiError::Invalid(errors))
    }
}
