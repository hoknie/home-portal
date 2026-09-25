use crate::scanner::scan;
use crate::sources::{SourceFile, rust_files};

pub const SUPPRESSIONS: [&str; 2] = ["#[allow(", "#![allow("];

pub fn violations(files: &[SourceFile]) -> Vec<String> {
    let mut found = Vec::new();
    for file in files {
        let code = scan(&file.text).code;
        for (index, line) in code.lines().enumerate() {
            let compact: String = line
                .chars()
                .filter(|current| !current.is_whitespace())
                .collect();
            if SUPPRESSIONS
                .iter()
                .any(|suppression| compact.contains(suppression))
            {
                found.push(format!("{}:{} suppresses a lint", file.path, index + 1));
            }
        }
    }
    found
}

#[test]
fn no_lint_is_suppressed() {
    let found = violations(&rust_files());
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn an_outer_and_an_inner_allow_are_both_caught() {
    let sample = SourceFile::sample(
        "crates/x/src/a.rs",
        "#![allow(unused)]\n#[allow(dead_code)]\nfn a() {}\n# [ allow (x)]\nfn b() {}\n",
    );
    assert_eq!(
        violations(&[sample]),
        vec![
            "crates/x/src/a.rs:1 suppresses a lint",
            "crates/x/src/a.rs:2 suppresses a lint",
            "crates/x/src/a.rs:4 suppresses a lint",
        ]
    );
}
