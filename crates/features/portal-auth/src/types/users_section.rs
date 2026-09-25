use portal_config::deserialize_section;
use serde::Deserialize;
use toml_edit::DocumentMut;

use super::User;

#[derive(Debug, Default, Deserialize)]
pub struct UsersSection {
    #[serde(default)]
    pub users: Vec<User>,
}

impl UsersSection {
    pub fn read(document: &DocumentMut) -> Result<UsersSection, String> {
        deserialize_section(document)
    }

    pub fn find(&self, name: &str) -> Option<&User> {
        self.users.iter().find(|user| user.name == name)
    }
}
