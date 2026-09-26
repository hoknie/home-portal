use std::fs;
use std::path::{Path, PathBuf};

use super::Directory;
use crate::ports::AssetSource;

fn interface(root: &Path) {
    fs::create_dir_all(root.join("en")).unwrap();
    fs::create_dir_all(root.join("_next/static")).unwrap();
    fs::write(root.join("en/index.html"), "home").unwrap();
    fs::write(root.join("_next/static/chunk.js"), "code").unwrap();
    fs::write(root.join("_next/static/blob.bin"), "bytes").unwrap();
}

#[test]
fn a_file_is_read_with_its_content_type_and_an_unknown_one_is_octet_stream() {
    let folder = tempfile::tempdir().unwrap();
    interface(folder.path());
    let directory = Directory::first_of(&[folder.path().to_path_buf()]);
    assert!(!directory.is_empty());
    let page = directory.get("en/index.html").unwrap();
    assert_eq!(page.bytes.as_ref(), b"home");
    assert_eq!(page.content_type, "text/html; charset=utf-8");
    assert_eq!(
        directory.get("_next/static/chunk.js").unwrap().content_type,
        "text/javascript"
    );
    assert_eq!(
        directory.get("_next/static/blob.bin").unwrap().content_type,
        "application/octet-stream"
    );
    assert!(directory.get("en/missing.html").is_none());
    assert!(directory.get("en").is_none());
}

#[test]
fn the_first_candidate_holding_an_entry_page_wins() {
    let empty = tempfile::tempdir().unwrap();
    let full = tempfile::tempdir().unwrap();
    interface(full.path());
    let missing = PathBuf::from("/nonexistent/web");
    let directory = Directory::first_of(&[
        missing,
        empty.path().to_path_buf(),
        full.path().to_path_buf(),
    ]);
    assert_eq!(
        directory.root(),
        Some(fs::canonicalize(full.path()).unwrap().as_path())
    );
}

#[test]
fn a_path_that_escapes_the_folder_reads_nothing() {
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("home-portal.toml"), "secret").unwrap();
    let folder = tempfile::tempdir().unwrap();
    interface(folder.path());
    std::os::unix::fs::symlink(outside.path(), folder.path().join("_next/outside")).unwrap();
    let directory = Directory::first_of(&[folder.path().to_path_buf()]);
    for path in [
        "_next/../../home-portal.toml",
        "../home-portal.toml",
        "_next/%2e%2e/%2e%2e/home-portal.toml",
        "_next/outside/home-portal.toml",
        "en//index.html",
        "en\\..\\index.html",
        "/etc/passwd",
    ] {
        assert!(directory.get(path).is_none(), "{path}");
    }
}

#[test]
fn without_an_entry_page_the_source_is_empty_and_names_where_it_looked() {
    let empty = tempfile::tempdir().unwrap();
    let directory = Directory::first_of(&[
        empty.path().to_path_buf(),
        PathBuf::from("/nonexistent/web"),
    ]);
    assert!(directory.is_empty());
    let message = directory.unavailable();
    assert!(
        message.contains(&empty.path().display().to_string()),
        "{message}"
    );
    assert!(message.contains("/nonexistent/web"), "{message}");
    assert!(message.contains("HOME_PORTAL_WEB"), "{message}");
}
