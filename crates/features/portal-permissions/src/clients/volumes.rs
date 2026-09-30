use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use super::bounded::run_bounded;
use super::folders::listed;
use crate::ports::PermissionCheck;
use crate::types::{Advice, Finding, PermissionCode, PermissionState, Ran};

pub const VOLUMES: &str = "/Volumes";
pub const DISKUTIL: &str = "/usr/sbin/diskutil";

pub type Removable = Arc<dyn Fn(&Path) -> bool + Send + Sync>;

pub struct VolumesCheck {
    pub root: PathBuf,
    pub system_device: Option<u64>,
    pub removable: Removable,
}

impl PermissionCheck for VolumesCheck {
    fn code(&self) -> PermissionCode {
        PermissionCode::RemovableVolumes
    }

    fn ask(&self) -> Finding {
        let volumes = self.volumes();
        if volumes.is_empty() {
            return Finding::not_applicable(Some(Advice::ConnectAVolume));
        }
        let mut finding = Finding::granted();
        for volume in volumes {
            let read = listed(&volume);
            match read.state {
                PermissionState::Denied => return read,
                PermissionState::Granted | PermissionState::NotApplicable => {}
                _ => finding = read,
            }
        }
        finding
    }
}

impl VolumesCheck {
    fn volumes(&self) -> Vec<PathBuf> {
        let Ok(entries) = fs::read_dir(&self.root) else {
            return Vec::new();
        };
        entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| !hidden(path))
            .filter(|path| {
                fs::metadata(path).is_ok_and(|metadata| {
                    metadata.is_dir() && Some(metadata.dev()) != self.system_device
                })
            })
            .filter(|path| (self.removable)(path))
            .collect()
    }
}

pub fn system_device() -> Option<u64> {
    system_device_of(Path::new("/"))
}

pub fn system_device_of(path: &Path) -> Option<u64> {
    fs::metadata(path).ok().map(|metadata| metadata.dev())
}

pub fn removable_by_diskutil(limit: Duration) -> Removable {
    Arc::new(move |path: &Path| {
        let mut command = Command::new(DISKUTIL);
        command.arg("info").arg("-plist").arg(path);
        match run_bounded(command, limit) {
            Ran::Finished(output) if output.status.success() => {
                removable_in(&String::from_utf8_lossy(&output.stdout))
            }
            _ => false,
        }
    })
}

pub fn removable_in(plist: &str) -> bool {
    flag(plist, "Internal") == Some(false)
        || flag(plist, "RemovableMedia") == Some(true)
        || flag(plist, "Ejectable") == Some(true)
}

fn flag(plist: &str, key: &str) -> Option<bool> {
    let marker = format!("<key>{key}</key>");
    let rest = plist[plist.find(&marker)? + marker.len()..].trim_start();
    if rest.starts_with("<true/>") {
        Some(true)
    } else if rest.starts_with("<false/>") {
        Some(false)
    } else {
        None
    }
}

fn hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}
