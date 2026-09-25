use std::fs;

use crate::sources::workspace_root;

pub const DOCUMENT: &str = "ARCHITECTURE.md";
pub const FENCE_OPENING: &str = "```exceptions";
pub const FENCE_CLOSING: &str = "```";

pub struct Exception {
    pub rule: String,
    pub path: String,
}

pub fn listed() -> Vec<Exception> {
    let text = fs::read_to_string(workspace_root().join(DOCUMENT))
        .expect("ARCHITECTURE.md exists at the workspace root");
    parse(&text).expect("ARCHITECTURE.md carries an exceptions block")
}

pub fn excepted(exceptions: &[Exception], rule: &str, path: &str) -> bool {
    exceptions
        .iter()
        .any(|exception| exception.rule == rule && exception.path == path)
}

pub fn parse(text: &str) -> Option<Vec<Exception>> {
    let mut lines = text.lines().skip_while(|line| line.trim() != FENCE_OPENING);
    lines.next()?;
    let exceptions = lines
        .take_while(|line| line.trim() != FENCE_CLOSING)
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            Some(Exception {
                rule: words.next()?.to_string(),
                path: words.next()?.to_string(),
            })
        })
        .collect();
    Some(exceptions)
}

#[test]
fn the_document_carries_an_exceptions_block() {
    listed();
}

#[test]
fn an_exception_is_a_rule_and_a_path() {
    let text = "intro\n```exceptions\nsize bin/a.rs\nroot crates/b/src/mod.rs\n```\nafter\n";
    let exceptions = parse(text).unwrap();
    assert!(excepted(&exceptions, "size", "bin/a.rs"));
    assert!(excepted(&exceptions, "root", "crates/b/src/mod.rs"));
    assert!(!excepted(&exceptions, "size", "crates/b/src/mod.rs"));
}

#[test]
fn a_document_without_the_block_is_refused() {
    assert!(parse("no block here").is_none());
}
