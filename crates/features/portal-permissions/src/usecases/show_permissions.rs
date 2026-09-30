use std::env::consts::OS;
use std::sync::Arc;

use super::ReadPermissionSettings;
use crate::ports::CheckSource;
use crate::services::{PermissionBoard, codes_of, view_of};
use crate::types::{Finding, Owner, PermissionCode, PermissionView, PermissionsView};

#[derive(Clone)]
pub struct ShowPermissions {
    settings: ReadPermissionSettings,
    board: PermissionBoard,
    source: Arc<dyn CheckSource>,
    owner: Owner,
}

impl ShowPermissions {
    pub fn new(
        settings: ReadPermissionSettings,
        board: PermissionBoard,
        source: Arc<dyn CheckSource>,
        owner: Owner,
    ) -> ShowPermissions {
        ShowPermissions {
            settings,
            board,
            source,
            owner,
        }
    }

    pub fn run(&self) -> PermissionsView {
        let applies = self.source.applies();
        let permissions = codes_of(&self.settings.run())
            .into_iter()
            .map(|code| {
                if applies {
                    let found = self.board.get(&code);
                    view_of(code, found)
                } else {
                    not_applicable(code)
                }
            })
            .collect();
        PermissionsView {
            platform: OS,
            owner: self.owner.clone(),
            permissions,
        }
    }
}

fn not_applicable(code: PermissionCode) -> PermissionView {
    let finding = Finding::not_applicable(None);
    PermissionView {
        code,
        state: finding.state,
        advice: finding.advice,
        learned_at: None,
    }
}
