use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProbeKind {
    #[default]
    Http,
    Tcp,
    Icmp,
}

impl ProbeKind {
    pub fn name(self) -> &'static str {
        match self {
            ProbeKind::Http => "http",
            ProbeKind::Tcp => "tcp",
            ProbeKind::Icmp => "icmp",
        }
    }

    pub fn speaks_http(self) -> bool {
        self == ProbeKind::Http
    }
}
