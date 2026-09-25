use std::fs;
use std::path::{Path, PathBuf};

pub const SCANNED_ROOTS: [&str; 2] = ["crates", "bin"];
pub const SKIPPED_DIRECTORIES: [&str; 2] = ["target", ".git"];

pub struct SourceFile {
    pub path: String,
    pub text: String,
}

impl SourceFile {
    pub fn sample(path: &str, text: &str) -> SourceFile {
        SourceFile {
            path: path.to_string(),
            text: text.to_string(),
        }
    }

    pub fn file_name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }
}

pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root exists")
}

pub fn directories() -> Vec<String> {
    let root = workspace_root();
    let mut found = Vec::new();
    for scanned in SCANNED_ROOTS {
        walk(&root, &root.join(scanned), &mut found);
    }
    found.sort();
    found
}

pub fn rust_files() -> Vec<SourceFile> {
    let root = workspace_root();
    let mut files: Vec<SourceFile> = Vec::new();
    for directory in directories() {
        let entries = fs::read_dir(root.join(&directory)).expect("a listed directory is readable");
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|extension| extension == "rs") {
                files.push(SourceFile {
                    path: relative(&root, &path),
                    text: fs::read_to_string(&path).expect("a source file is utf-8"),
                });
            }
        }
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    files
}

fn walk(root: &Path, directory: &Path, found: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    found.push(relative(root, directory));
    for entry in entries.flatten() {
        let path = entry.path();
        let skipped = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| SKIPPED_DIRECTORIES.contains(&name));
        if path.is_dir() && !skipped {
            walk(root, &path, found);
        }
    }
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[test]
fn the_walk_finds_the_binary_and_skips_build_output() {
    let paths: Vec<String> = rust_files().into_iter().map(|file| file.path).collect();
    assert!(paths.contains(&"bin/home-portal/src/main.rs".to_string()));
    assert!(paths.iter().all(|path| !path.contains("/target/")));
}
