use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use portal_web::{AssetSource, Directory, WEB_VARIABLE};

pub const BESIDE_THE_BINARY: &str = "web";
pub const SHARED_FOLDER: &str = "share/home-portal/web";

pub fn interface_folder(
    web_variable: Option<OsString>,
    executable: Option<PathBuf>,
) -> Vec<PathBuf> {
    if let Some(folder) = web_variable.filter(|value| !value.is_empty()) {
        return vec![PathBuf::from(folder)];
    }
    let Some(directory) = executable
        .and_then(|path| fs::canonicalize(path).ok())
        .and_then(|path| path.parent().map(PathBuf::from))
    else {
        return Vec::new();
    };
    let mut candidates = vec![directory.join(BESIDE_THE_BINARY)];
    if let Some(prefix) = directory.parent() {
        candidates.push(prefix.join(SHARED_FOLDER));
    }
    candidates
}

pub fn located() -> Arc<dyn AssetSource> {
    let candidates = interface_folder(env::var_os(WEB_VARIABLE), env::current_exe().ok());
    let directory = Directory::first_of(&candidates);
    match directory.root() {
        Some(root) => tracing::info!(folder = %root.display(), "serving the interface"),
        None => tracing::warn!("{}", directory.unavailable()),
    }
    Arc::new(directory)
}
