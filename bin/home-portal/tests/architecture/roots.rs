use syn::Item;

use crate::sources::{SourceFile, rust_files};

pub const ROOT_FILES: [&str; 2] = ["lib.rs", "mod.rs"];

pub fn violations(files: &[SourceFile]) -> Vec<String> {
    files
        .iter()
        .filter(|file| ROOT_FILES.contains(&file.file_name()))
        .filter_map(|file| match syn::parse_file(&file.text) {
            Err(error) => Some(format!("{} does not parse: {error}", file.path)),
            Ok(parsed) => parsed
                .items
                .iter()
                .any(|item| !declaration(item))
                .then(|| format!("{} holds more than mod and use declarations", file.path)),
        })
        .collect()
}

fn declaration(item: &Item) -> bool {
    match item {
        Item::Use(_) => true,
        Item::Mod(module) => module.content.is_none(),
        _ => false,
    }
}

#[test]
fn module_roots_hold_declarations_only() {
    let found = violations(&rust_files());
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_function_in_lib_rs_is_caught_by_file() {
    let sample = SourceFile::sample(
        "crates/x/src/lib.rs",
        "mod a;\npub use a::A;\nfn helper() {}\n",
    );
    assert_eq!(
        violations(&[sample]),
        vec!["crates/x/src/lib.rs holds more than mod and use declarations"]
    );
}

#[test]
fn an_inline_module_body_is_not_a_declaration() {
    let sample = SourceFile::sample("crates/x/src/mod.rs", "mod a { pub struct A; }\n");
    assert_eq!(violations(&[sample]).len(), 1);
}

#[test]
fn attributes_on_declarations_are_allowed() {
    let sample = SourceFile::sample(
        "crates/x/src/mod.rs",
        "mod a;\n#[cfg(test)]\nmod tests;\npub use a::A;\n",
    );
    assert!(violations(&[sample]).is_empty());
}
