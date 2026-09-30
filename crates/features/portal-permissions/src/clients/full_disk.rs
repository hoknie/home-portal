use std::fs::File;
use std::io::ErrorKind;
use std::path::PathBuf;

use crate::ports::PermissionCheck;
use crate::types::{Advice, Finding, PermissionCode};

pub const PROTECTED_FILE: &str = "Library/Application Support/com.apple.TCC/TCC.db";

pub struct FullDiskCheck {
    pub path: PathBuf,
}

impl PermissionCheck for FullDiskCheck {
    fn code(&self) -> PermissionCode {
        PermissionCode::FullDiskAccess
    }

    fn ask(&self) -> Finding {
        match File::open(&self.path) {
            Ok(_) => Finding::granted(),
            Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                Finding::denied(Advice::GrantFullDiskAccess)
            }
            Err(_) => Finding::unknown(Advice::CheckFailed),
        }
    }
}
