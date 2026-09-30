#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Action {
    Read,
    Create,
    Update,
    Delete,
    Execute,
}

impl Action {
    pub const ALL: [Action; 5] = [
        Action::Read,
        Action::Create,
        Action::Update,
        Action::Delete,
        Action::Execute,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Create => "create",
            Action::Update => "update",
            Action::Delete => "delete",
            Action::Execute => "execute",
        }
    }

    pub fn from_name(name: &str) -> Option<Action> {
        Action::ALL.into_iter().find(|action| action.name() == name)
    }
}
