use toml_edit::DocumentMut;

use crate::types::UsersSection;

pub fn user_names(document: &DocumentMut) -> Vec<String> {
    UsersSection::read(document)
        .map(|section| section.users.into_iter().map(|user| user.name).collect())
        .unwrap_or_default()
}
