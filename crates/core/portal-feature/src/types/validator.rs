use toml_edit::DocumentMut;

use super::FieldError;

pub type Validator = fn(&DocumentMut) -> Vec<FieldError>;
