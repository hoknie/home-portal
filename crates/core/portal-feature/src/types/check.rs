use std::sync::Arc;

use toml_edit::DocumentMut;

use super::FieldError;

pub type Check = Arc<dyn Fn(&DocumentMut) -> Vec<FieldError> + Send + Sync>;
