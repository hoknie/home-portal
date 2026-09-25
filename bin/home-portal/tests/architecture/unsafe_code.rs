use crate::scanner::scan;
use crate::sources::{SourceFile, rust_files};

pub const KEYWORD: &str = "unsafe";
pub const PERMITTED: &str = "crates/features/portal-automations/src/clients/process_group.rs";

pub fn violations(files: &[SourceFile]) -> Vec<String> {
    let mut found = Vec::new();
    for file in files.iter().filter(|file| file.path != PERMITTED) {
        let code = scan(&file.text).code;
        for (index, line) in code.lines().enumerate() {
            let used = line
                .split(|current: char| !(current.is_alphanumeric() || current == '_'))
                .any(|word| word == KEYWORD);
            if used {
                found.push(format!("{}:{} uses unsafe code", file.path, index + 1));
            }
        }
    }
    found
}

#[test]
fn unsafe_code_lives_only_in_the_process_group_file() {
    let found = violations(&rust_files());
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_stray_unsafe_block_is_caught_and_the_word_in_text_is_not() {
    let stray = SourceFile::sample(
        "crates/x/src/a.rs",
        "fn a() {\n    unsafe { b() }\n}\nconst C: &str = \"unsafe\";\nfn unsafe_looking() {}\n",
    );
    let permitted = SourceFile::sample(PERMITTED, "fn a() { unsafe { b() } }\n");
    assert_eq!(
        violations(&[stray, permitted]),
        vec!["crates/x/src/a.rs:2 uses unsafe code"]
    );
}
