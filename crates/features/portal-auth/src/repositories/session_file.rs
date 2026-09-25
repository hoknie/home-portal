use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::types::{Session, SessionsFileBody};

pub struct SessionFile {
    path: PathBuf,
}

impl SessionFile {
    pub const BROKEN_SUFFIX: &'static str = ".broken";
    pub const TEMPORARY_SUFFIX: &'static str = ".tmp";
    pub const PRIVATE_MODE: u32 = 0o600;

    pub fn at(path: PathBuf) -> SessionFile {
        SessionFile { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> HashMap<String, Session> {
        let Ok(text) = fs::read_to_string(&self.path) else {
            return HashMap::new();
        };
        match serde_json::from_str::<SessionsFileBody>(&text) {
            Ok(body) => body.sessions,
            Err(error) => {
                let mut broken = self.path.clone().into_os_string();
                broken.push(Self::BROKEN_SUFFIX);
                tracing::warn!(path = %self.path.display(), %error, "the sessions file cannot be read; it is set aside and everyone signs in again");
                if let Err(error) = fs::rename(&self.path, &broken) {
                    tracing::warn!(path = %self.path.display(), %error, "cannot set the sessions file aside");
                }
                HashMap::new()
            }
        }
    }

    pub fn save(&self, sessions: &HashMap<String, Session>) -> io::Result<()> {
        let body = SessionsFileBody {
            sessions: sessions.clone(),
        };
        let text = serde_json::to_string(&body).map_err(io::Error::other)?;
        let mut temporary = self.path.clone().into_os_string();
        temporary.push(Self::TEMPORARY_SUFFIX);
        let temporary = PathBuf::from(temporary);
        let mut file = Self::open_private(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary, &self.path)
    }

    #[cfg(unix)]
    fn open_private(path: &Path) -> io::Result<fs::File> {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(Self::PRIVATE_MODE)
            .open(path)?;
        file.set_permissions(fs::Permissions::from_mode(Self::PRIVATE_MODE))?;
        Ok(file)
    }

    #[cfg(not(unix))]
    fn open_private(path: &Path) -> io::Result<fs::File> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
    }
}
