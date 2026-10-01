use portal_permissions::{
    Advice, OwnerKind, OwnerResponse, Pane, PermissionResponse, PermissionState,
    PermissionsResponse,
};
use time::macros::datetime;

use crate::typed;

fn permission(
    code: &str,
    state: PermissionState,
    advice: Option<Advice>,
    pane: Pane,
) -> PermissionResponse {
    PermissionResponse {
        code: code.to_string(),
        state,
        learned_at: Some(datetime!(2026-09-29 19:00:00 UTC)),
        advice,
        pane,
    }
}

#[test]
fn the_permissions_sample_matches_its_serializer() {
    let permissions = PermissionsResponse {
        platform: "macos".to_string(),
        owner: OwnerResponse {
            kind: OwnerKind::Binary,
            name: "/usr/local/bin/home-portal".to_string(),
        },
        permissions: vec![
            permission(
                "local-network",
                PermissionState::Granted,
                None,
                Pane::LocalNetwork,
            ),
            permission(
                "removable-volumes",
                PermissionState::Denied,
                Some(Advice::AllowInSettings),
                Pane::FilesAndFolders,
            ),
            permission(
                "folder:Documents",
                PermissionState::Pending,
                Some(Advice::AnswerThePrompt),
                Pane::FilesAndFolders,
            ),
            permission(
                "folder:Desktop",
                PermissionState::NotApplicable,
                None,
                Pane::FilesAndFolders,
            ),
            permission(
                "automation:System Events",
                PermissionState::Unknown,
                Some(Advice::ApplicationNotFound),
                Pane::Automation,
            ),
            PermissionResponse {
                code: "full-disk-access".to_string(),
                state: PermissionState::Unknown,
                learned_at: None,
                advice: None,
                pane: Pane::FullDiskAccess,
            },
        ],
    };
    typed("permissions", &permissions);
}
