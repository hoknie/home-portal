use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceState {
    Unknown,
    Up,
    Degraded,
    Down,
    Unreadable,
}

impl ServiceState {
    pub const ALL: [ServiceState; 5] = [
        ServiceState::Unknown,
        ServiceState::Up,
        ServiceState::Degraded,
        ServiceState::Down,
        ServiceState::Unreadable,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ServiceState::Unknown => "unknown",
            ServiceState::Up => "up",
            ServiceState::Degraded => "degraded",
            ServiceState::Down => "down",
            ServiceState::Unreadable => "unreadable",
        }
    }

    pub fn answered(self) -> bool {
        matches!(self, ServiceState::Up | ServiceState::Degraded)
    }

    pub fn failed(self) -> bool {
        matches!(self, ServiceState::Down | ServiceState::Unreadable)
    }
}
