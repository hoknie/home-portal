use crate::types::{Finding, PermissionCode};

pub trait PermissionCheck: Send + Sync {
    fn code(&self) -> PermissionCode;

    fn ask(&self) -> Finding;
}
