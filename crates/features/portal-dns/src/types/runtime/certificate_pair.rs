use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificatePair {
    pub certificate: PathBuf,
    pub key: PathBuf,
    pub modified: Option<(SystemTime, SystemTime)>,
}
