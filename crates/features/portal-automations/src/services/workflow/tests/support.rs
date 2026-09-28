use toml_edit::DocumentMut;

use crate::services::workflow::workflow_errors;
use crate::types::AutomationsSection;

pub fn section(text: &str) -> AutomationsSection {
    AutomationsSection::read(&text.parse::<DocumentMut>().unwrap()).unwrap()
}

pub fn fields(text: &str) -> Vec<String> {
    workflow_errors(&section(text))
        .into_iter()
        .map(|error| error.field)
        .collect()
}
