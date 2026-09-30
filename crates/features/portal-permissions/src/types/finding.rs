use super::{Advice, PermissionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Finding {
    pub state: PermissionState,
    pub advice: Option<Advice>,
}

impl Finding {
    pub fn granted() -> Finding {
        Finding {
            state: PermissionState::Granted,
            advice: None,
        }
    }

    pub fn denied(advice: Advice) -> Finding {
        Finding {
            state: PermissionState::Denied,
            advice: Some(advice),
        }
    }

    pub fn pending() -> Finding {
        Finding {
            state: PermissionState::Pending,
            advice: Some(Advice::AnswerThePrompt),
        }
    }

    pub fn not_applicable(advice: Option<Advice>) -> Finding {
        Finding {
            state: PermissionState::NotApplicable,
            advice,
        }
    }

    pub fn unknown(advice: Advice) -> Finding {
        Finding {
            state: PermissionState::Unknown,
            advice: Some(advice),
        }
    }
}
