use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Diagnosis {
    LocalNetworkDenied,
    Refused,
    Timeout,
    HostUnreachable,
    NameNotResolved,
    Tls,
    NotHttp,
    HttpStatus,
    IcmpNotPermitted,
    Other,
    #[serde(other)]
    Unknown,
}

impl Diagnosis {
    pub const ALL: [Diagnosis; 10] = [
        Diagnosis::LocalNetworkDenied,
        Diagnosis::Refused,
        Diagnosis::Timeout,
        Diagnosis::HostUnreachable,
        Diagnosis::NameNotResolved,
        Diagnosis::Tls,
        Diagnosis::NotHttp,
        Diagnosis::HttpStatus,
        Diagnosis::IcmpNotPermitted,
        Diagnosis::Other,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Diagnosis::LocalNetworkDenied => "local-network-denied",
            Diagnosis::Refused => "refused",
            Diagnosis::Timeout => "timeout",
            Diagnosis::HostUnreachable => "host-unreachable",
            Diagnosis::NameNotResolved => "name-not-resolved",
            Diagnosis::Tls => "tls",
            Diagnosis::NotHttp => "not-http",
            Diagnosis::HttpStatus => "http-status",
            Diagnosis::IcmpNotPermitted => "icmp-not-permitted",
            Diagnosis::Other => "other",
            Diagnosis::Unknown => "unknown",
        }
    }
}
