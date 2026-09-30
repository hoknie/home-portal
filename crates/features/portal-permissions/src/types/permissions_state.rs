use crate::usecases::{RequestPermissions, ShowPermissions};

#[derive(Clone)]
pub struct PermissionsState {
    pub show: ShowPermissions,
    pub request: RequestPermissions,
}
