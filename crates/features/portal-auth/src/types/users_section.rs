use portal_config::deserialize_section;
use serde::Deserialize;
use toml_edit::DocumentMut;

use super::{Group, User};
use crate::helpers::credential_of;

#[derive(Debug, Default, Deserialize)]
pub struct UsersSection {
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub groups: Vec<Group>,
}

impl UsersSection {
    pub fn read(document: &DocumentMut) -> Result<UsersSection, String> {
        deserialize_section(document)
    }

    pub fn find(&self, name: &str) -> Option<&User> {
        self.users.iter().find(|user| user.name == name)
    }

    pub fn group(&self, name: &str) -> Option<&Group> {
        self.groups.iter().find(|group| group.name == name)
    }

    pub fn credential(&self, name: &str) -> Option<String> {
        self.find(name)
            .map(|user| credential_of(&user.password_hash))
    }
}
