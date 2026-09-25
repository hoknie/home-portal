mod edit_validation;
mod layout;

#[cfg(test)]
mod tests;

pub use edit_validation::{check_edited, renamed_for_the_editor};
pub use layout::{layout, validate_dashboard};
