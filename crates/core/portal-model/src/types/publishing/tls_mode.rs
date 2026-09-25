use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum TlsMode {
    #[default]
    Acme,
    Internal,
    Files,
}

impl TlsMode {
    pub fn name(self) -> &'static str {
        match self {
            TlsMode::Acme => "acme",
            TlsMode::Internal => "internal",
            TlsMode::Files => "files",
        }
    }
}
