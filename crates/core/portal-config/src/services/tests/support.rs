use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use portal_feature::FieldError;
use tempfile::TempDir;
use toml_edit::DocumentMut;

use crate::services::ConfigStore;
use crate::types::ConfigError;

pub const COMMENTED: &str = "# my services, edited by hand\n\n# the media box\n[[services]]\nid = \"media\"\nname = \"Media\"\n";

pub fn no_negative_ports(document: &DocumentMut) -> Vec<FieldError> {
    match document.get("port").and_then(|item| item.as_integer()) {
        Some(port) if port < 0 => vec![FieldError::new("port", "must not be negative")],
        _ => Vec::new(),
    }
}

pub fn services_have_names(document: &DocumentMut) -> Vec<FieldError> {
    document
        .get("services")
        .and_then(|item| item.as_array_of_tables())
        .into_iter()
        .flat_map(|entries| entries.iter().enumerate())
        .filter(|(_, table)| table.get("name").is_none())
        .map(|(index, _)| FieldError::new(format!("services[{index}].name"), "is required"))
        .collect()
}

pub struct Portal {
    pub directory: TempDir,
    pub store: Arc<ConfigStore>,
}

impl Portal {
    pub fn with(files: &[(&str, &str)]) -> Portal {
        let directory = tempfile::tempdir().unwrap();
        for (name, text) in files {
            let path = directory.path().join(name);
            fs::write(&path, text).unwrap();
            if text.contains("[secrets]") {
                private(&path);
            }
        }
        let store = Arc::new(ConfigStore::open(directory.path().join(files[0].0)).unwrap());
        Portal { directory, store }
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.directory.path().join(name)
    }

    pub fn text(&self, name: &str) -> String {
        fs::read_to_string(self.path(name)).unwrap()
    }
}

pub fn open(files: &[(&str, &str)]) -> Result<ConfigStore, ConfigError> {
    let directory = Box::leak(Box::new(tempfile::tempdir().unwrap()));
    for (name, text) in files {
        fs::write(directory.path().join(name), text).unwrap();
    }
    ConfigStore::open(directory.path().join(files[0].0))
}

pub fn rewrite(path: &Path, text: &str) {
    let before = fs::metadata(path).unwrap().modified().unwrap();
    fs::write(path, text).unwrap();
    let file = fs::File::options().write(true).open(path).unwrap();
    file.set_modified(before + std::time::Duration::from_secs(2))
        .unwrap();
}

pub fn private(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }
}
