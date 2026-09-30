use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use super::codes::codes_of;
use crate::clients::{
    AutomationCheck, FolderCheck, FullDiskCheck, LocalNetworkCheck, NotApplicableCheck, OSASCRIPT,
    PROTECTED_FILE, VOLUMES, VolumesCheck, multicast, removable_by_diskutil, system_device,
};
use crate::ports::{CheckSource, PermissionCheck};
use crate::types::{Limits, PermissionCode, PermissionSettings};

pub const MACOS: bool = cfg!(target_os = "macos");

pub struct HostChecks {
    pub macos: bool,
    pub home: Option<PathBuf>,
    pub limits: Limits,
}

impl HostChecks {
    pub fn of_this_host() -> HostChecks {
        HostChecks {
            macos: MACOS,
            home: env::var_os("HOME")
                .filter(|home| !home.is_empty())
                .map(PathBuf::from),
            limits: Limits::default(),
        }
    }

    fn check(&self, code: PermissionCode) -> Arc<dyn PermissionCheck> {
        let home = self.home.clone();
        match (code, home) {
            (PermissionCode::LocalNetwork, _) => Arc::new(LocalNetworkCheck {
                send: multicast(),
                limit: self.limits.network,
                retry: self.limits.retry,
            }),
            (PermissionCode::RemovableVolumes, _) => Arc::new(VolumesCheck {
                root: PathBuf::from(VOLUMES),
                system_device: system_device(),
                removable: removable_by_diskutil(self.limits.diskutil),
            }),
            (PermissionCode::Folder(name), Some(home)) => Arc::new(FolderCheck {
                path: home.join(&name),
                name,
            }),
            (PermissionCode::Automation(application), _) => Arc::new(AutomationCheck {
                application,
                program: PathBuf::from(OSASCRIPT),
                limit: self.limits.prompt,
            }),
            (PermissionCode::FullDiskAccess, Some(home)) => Arc::new(FullDiskCheck {
                path: home.join(PROTECTED_FILE),
            }),
            (code, None) => Arc::new(NotApplicableCheck { code }),
        }
    }
}

impl CheckSource for HostChecks {
    fn applies(&self) -> bool {
        self.macos
    }

    fn checks(&self, settings: &PermissionSettings) -> Vec<Arc<dyn PermissionCheck>> {
        codes_of(settings)
            .into_iter()
            .map(|code| {
                if self.macos {
                    self.check(code)
                } else {
                    Arc::new(NotApplicableCheck { code })
                }
            })
            .collect()
    }
}
