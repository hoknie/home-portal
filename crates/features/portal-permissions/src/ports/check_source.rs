use std::sync::Arc;

use super::PermissionCheck;
use crate::types::PermissionSettings;

pub trait CheckSource: Send + Sync {
    fn applies(&self) -> bool;

    fn checks(&self, settings: &PermissionSettings) -> Vec<Arc<dyn PermissionCheck>>;
}
