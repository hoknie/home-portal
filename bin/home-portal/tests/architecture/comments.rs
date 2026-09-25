use crate::scanner::scan;
use crate::sources::{SourceFile, rust_files};

pub fn violations(files: &[SourceFile]) -> Vec<String> {
    files
        .iter()
        .flat_map(|file| {
            scan(&file.text)
                .comment_lines
                .into_iter()
                .map(move |line| format!("{}:{line} holds a comment", file.path))
        })
        .collect()
}

#[test]
fn there_are_no_comments_in_rust_sources() {
    let found = violations(&rust_files());
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_doc_comment_is_caught_with_its_file_and_line() {
    let sample = SourceFile::sample("crates/x/src/a.rs", "fn a() {}\n/// doc\nfn b() {}\n");
    assert_eq!(
        violations(&[sample]),
        vec!["crates/x/src/a.rs:2 holds a comment"]
    );
}

#[test]
fn a_url_in_a_string_passes() {
    let sample = SourceFile::sample(
        "crates/x/src/a.rs",
        "const A: &str = \"http://127.0.0.1:8080\";\n",
    );
    assert!(violations(&[sample]).is_empty());
}
