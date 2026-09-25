use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub const PREVIOUS_SUFFIX: &str = ".previous";
pub const TEMPORARY_SUFFIX: &str = ".tmp";
pub const PRIVATE_MODE: u32 = 0o600;

pub fn write_atomically(path: &Path, previous: &[u8], contents: &str) -> io::Result<()> {
    let temporary = sibling(path, TEMPORARY_SUFFIX);
    let mut file = open_private(&temporary)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    let mut kept = open_private(&sibling(path, PREVIOUS_SUFFIX))?;
    kept.write_all(previous)?;
    kept.sync_all()?;
    fs::rename(&temporary, path)
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

#[cfg(unix)]
fn open_private(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(PRIVATE_MODE)
        .open(path)?;
    file.set_permissions(fs::Permissions::from_mode(PRIVATE_MODE))?;
    Ok(file)
}

#[cfg(not(unix))]
fn open_private(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
}
