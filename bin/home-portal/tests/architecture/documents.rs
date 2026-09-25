use std::fs;

use crate::sources::workspace_root;

pub const ENGLISH: &str = "ARCHITECTURE.md";
pub const RUSSIAN: &str = "ARCHITECTURE-RU.md";

fn read(name: &str) -> String {
    fs::read_to_string(workspace_root().join(name)).expect("the document exists at the root")
}

pub fn headings(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.starts_with("## ") || line.starts_with("### "))
        .map(|line| {
            line.split_whitespace()
                .nth(1)
                .unwrap_or_default()
                .trim_end_matches('.')
                .to_string()
        })
        .collect()
}

pub const EXTENSIONS: [&str; 6] = [".rs", ".ts", ".tsx", ".toml", ".json", ".md"];
pub const SKIPPED: [&str; 5] = ["target", ".git", "node_modules", "out", ".next"];

pub fn quoted_paths(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (index, part) in text.split('`').enumerate() {
        if index % 2 == 0
            || !part.contains('/')
            || part.contains('<')
            || part.contains(' ')
            || part.contains("YYYY")
        {
            continue;
        }
        let candidate = part.trim_start_matches("./").trim_end_matches('/');
        let named = ["crates", "bin", "web", "docs"]
            .iter()
            .any(|root| candidate.starts_with(root));
        if named
            || EXTENSIONS
                .iter()
                .any(|extension| candidate.ends_with(extension))
        {
            found.push(candidate.to_string());
        }
    }
    found
}

fn every_path(root: &std::path::Path, prefix: &str, into: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if SKIPPED.contains(&name.as_str()) {
            continue;
        }
        let path = format!("{prefix}{name}");
        if entry.path().is_dir() {
            into.push(path.clone());
            every_path(&entry.path(), &format!("{path}/"), into);
        } else {
            into.push(path);
        }
    }
}

#[test]
fn both_language_copies_have_the_same_sections_in_the_same_order() {
    assert_eq!(headings(&read(ENGLISH)), headings(&read(RUSSIAN)));
}

#[test]
fn every_path_the_documents_name_exists() {
    let root = workspace_root();
    let mut present = Vec::new();
    every_path(&root, "", &mut present);
    for name in [ENGLISH, RUSSIAN] {
        for path in quoted_paths(&read(name)) {
            assert!(
                present
                    .iter()
                    .any(|known| known == &path || known.ends_with(&format!("/{path}"))),
                "{name} names {path}, which does not exist"
            );
        }
    }
}

#[test]
fn the_guard_reads_headings_and_paths_and_ignores_prose() {
    let sample = "## 2. Topology\n\n`crates/core/portal-model/` and `portal-<name>/` and `a b/c` and `docs/designs/YYYY-MM-DD-DESIGN-slug.md` and `adapters/public_layout.rs`\n### 2.1. Layers\n";
    assert_eq!(headings(sample), vec!["2", "2.1"]);
    assert_eq!(
        quoted_paths(sample),
        vec!["crates/core/portal-model", "adapters/public_layout.rs"]
    );
}
