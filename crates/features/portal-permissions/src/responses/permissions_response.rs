use serde::Serialize;

use super::{OwnerResponse, PermissionResponse};
use crate::types::PermissionsView;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PermissionsResponse {
    pub platform: String,
    pub owner: OwnerResponse,
    pub permissions: Vec<PermissionResponse>,
}

impl PermissionsResponse {
    pub fn of(view: &PermissionsView) -> PermissionsResponse {
        PermissionsResponse {
            platform: view.platform.to_string(),
            owner: OwnerResponse::of(&view.owner),
            permissions: view
                .permissions
                .iter()
                .map(PermissionResponse::of)
                .collect(),
        }
    }
}
