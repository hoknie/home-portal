use super::{Right, Rights};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    Signed,
    AnyOf(&'static [Right]),
    Admin,
}

impl Requirement {
    pub fn met_by(self, rights: &Rights) -> bool {
        match self {
            Requirement::Signed => true,
            Requirement::AnyOf(needed) => needed.iter().any(|right| rights.allows(*right)),
            Requirement::Admin => rights.is_admin(),
        }
    }

    pub fn describe(self) -> String {
        match self {
            Requirement::Signed => "a session".to_string(),
            Requirement::AnyOf(needed) => needed
                .iter()
                .map(Right::to_string)
                .collect::<Vec<_>>()
                .join(" or "),
            Requirement::Admin => "membership of admin".to_string(),
        }
    }
}
