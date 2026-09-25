use std::fs;
use std::path::Path;

use crate::types::ConfigError;

#[cfg(unix)]
pub fn refuse_if_readable(path: &Path) -> Result<(), ConfigError> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = fs::metadata(path).map_err(|source| ConfigError::Unreadable {
        path: path.to_path_buf(),
        source,
    })?;
    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(ConfigError::Permissions {
            path: path.to_path_buf(),
            mode,
        });
    }
    Ok(())
}

#[cfg(not(unix))]
pub fn refuse_if_readable(_path: &Path) -> Result<(), ConfigError> {
    Ok(())
}
