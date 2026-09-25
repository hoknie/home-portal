use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use home_portal::Wiring;
use portal_auth::hash_password;
use portal_config::ConfigStore;
use portal_network::EffectiveAddress;
use tempfile::TempDir;

pub const EXAMPLE: &str = "config/home-portal.example.toml";

pub fn with_extra(directory: &TempDir, password: &str, extra: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let example = fs::read_to_string(root.join(EXAMPLE)).unwrap();
    let hash = hash_password(password).unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(
        &path,
        format!("{example}\n[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\n{extra}"),
    )
    .unwrap();
    path
}

pub fn wiring_for(path: &Path) -> Wiring {
    Wiring {
        configuration: Arc::new(ConfigStore::open(path).unwrap()),
        effective: EffectiveAddress {
            address: "127.0.0.1:8080".parse().unwrap(),
            overridden: false,
        },
    }
}
