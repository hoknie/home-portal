use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;

pub const MAIN_FILE: &str = "home-portal.toml";

pub fn written(main: &str) -> (TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(MAIN_FILE);
    fs::write(&path, main).unwrap();
    (directory, path)
}
