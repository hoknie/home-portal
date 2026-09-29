use std::fs;
use std::os::unix::fs::MetadataExt;

use crate::ports::ProcessIdentity;

pub struct FileOwner {
    user: u32,
    group: u32,
}

impl FileOwner {
    pub fn of_this_process() -> FileOwner {
        let probe = tempfile::NamedTempFile::new().unwrap();
        let metadata = fs::metadata(probe.path()).unwrap();
        FileOwner {
            user: metadata.uid(),
            group: metadata.gid(),
        }
    }
}

impl ProcessIdentity for FileOwner {
    fn user(&self) -> u32 {
        self.user
    }

    fn groups(&self) -> Vec<u32> {
        vec![self.group]
    }
}
