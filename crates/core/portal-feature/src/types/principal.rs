use super::Rights;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    pub name: String,
    pub group: Option<String>,
    pub rights: Rights,
}

impl Principal {
    pub const ADMIN: &'static str = "admin";

    pub fn admin(name: impl Into<String>) -> Principal {
        Principal {
            name: name.into(),
            group: Some(Self::ADMIN.to_string()),
            rights: Rights::admin(),
        }
    }

    pub fn member(name: impl Into<String>, group: Option<String>, rights: Rights) -> Principal {
        Principal {
            name: name.into(),
            group,
            rights,
        }
    }
}
