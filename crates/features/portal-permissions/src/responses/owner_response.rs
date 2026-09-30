use serde::Serialize;

use crate::types::{Owner, OwnerKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OwnerResponse {
    pub kind: OwnerKind,
    pub name: String,
}

impl OwnerResponse {
    pub fn of(owner: &Owner) -> OwnerResponse {
        OwnerResponse {
            kind: owner.kind,
            name: owner.name.clone(),
        }
    }
}
