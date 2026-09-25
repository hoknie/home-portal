use std::collections::BTreeMap;

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::sources::{SourceFile, rust_files};

pub const FILE_LINES: usize = 400;
pub const FUNCTION_LINES: usize = 300;
pub const FOLDER_FILES: usize = 12;
pub const UNCOUNTED_FILES: [&str; 4] = ["mod.rs", "tests.rs", "lib.rs", "main.rs"];

pub fn file_violations(files: &[SourceFile]) -> Vec<String> {
    files
        .iter()
        .filter_map(|file| {
            let lines = file.text.lines().count();
            (lines > FILE_LINES)
                .then(|| format!("{} has {lines} lines, above {FILE_LINES}", file.path))
        })
        .collect()
}

pub fn function_violations(files: &[SourceFile]) -> Vec<String> {
    let mut found = Vec::new();
    for file in files {
        let Ok(parsed) = syn::parse_file(&file.text) else {
            continue;
        };
        let mut lengths = FunctionLengths::default();
        lengths.visit_file(&parsed);
        for (name, lines) in lengths.found {
            if lines > FUNCTION_LINES {
                found.push(format!(
                    "{}: fn {name} has {lines} lines, above {FUNCTION_LINES}",
                    file.path
                ));
            }
        }
    }
    found
}

pub fn folder_violations(files: &[SourceFile]) -> Vec<String> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for file in files {
        if UNCOUNTED_FILES.contains(&file.file_name()) {
            continue;
        }
        let folder = file.path.rsplit_once('/').map_or("", |(folder, _)| folder);
        *counts.entry(folder).or_default() += 1;
    }
    counts
        .into_iter()
        .filter(|(_, count)| *count > FOLDER_FILES)
        .map(|(folder, count)| format!("{folder} holds {count} files, above {FOLDER_FILES}"))
        .collect()
}

#[derive(Default)]
struct FunctionLengths {
    found: Vec<(String, usize)>,
}

impl FunctionLengths {
    fn record(&mut self, name: String, span: Span) {
        let lines = span.end().line - span.start().line + 1;
        self.found.push((name, lines));
    }
}

impl<'syntax> Visit<'syntax> for FunctionLengths {
    fn visit_item_fn(&mut self, function: &'syntax syn::ItemFn) {
        self.record(function.sig.ident.to_string(), function.span());
        syn::visit::visit_item_fn(self, function);
    }

    fn visit_impl_item_fn(&mut self, function: &'syntax syn::ImplItemFn) {
        self.record(function.sig.ident.to_string(), function.span());
        syn::visit::visit_impl_item_fn(self, function);
    }

    fn visit_trait_item_fn(&mut self, function: &'syntax syn::TraitItemFn) {
        self.record(function.sig.ident.to_string(), function.span());
        syn::visit::visit_trait_item_fn(self, function);
    }
}

fn lines_of_code(count: usize) -> String {
    (0..count)
        .map(|index| format!("const C{index}: u8 = 0;\n"))
        .collect()
}

fn function_of(lines: usize) -> String {
    let body: String = (0..lines - 2)
        .map(|index| format!("    let _v{index} = 0;\n"))
        .collect();
    format!("fn long() {{\n{body}}}\n")
}

#[test]
fn every_file_function_and_folder_is_within_budget() {
    let files = rust_files();
    let mut found = file_violations(&files);
    found.extend(function_violations(&files));
    found.extend(folder_violations(&files));
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_file_of_four_hundred_lines_passes_and_one_more_fails_by_name() {
    let fitting = SourceFile::sample("crates/x/src/a.rs", &lines_of_code(FILE_LINES));
    let oversized = SourceFile::sample("crates/x/src/b.rs", &lines_of_code(FILE_LINES + 1));
    assert_eq!(
        file_violations(&[fitting, oversized]),
        vec!["crates/x/src/b.rs has 401 lines, above 400"]
    );
}

#[test]
fn a_function_of_three_hundred_lines_passes_and_one_more_fails() {
    let fitting = SourceFile::sample("crates/x/src/a.rs", &function_of(FUNCTION_LINES));
    let oversized = SourceFile::sample("crates/x/src/b.rs", &function_of(FUNCTION_LINES + 1));
    assert_eq!(
        function_violations(&[fitting, oversized]),
        vec!["crates/x/src/b.rs: fn long has 301 lines, above 300"]
    );
}

#[test]
fn a_folder_of_thirteen_counted_files_fails() {
    let mut files: Vec<SourceFile> = (0..=FOLDER_FILES)
        .map(|index| SourceFile::sample(&format!("crates/x/src/rules/r{index}.rs"), ""))
        .collect();
    files.push(SourceFile::sample("crates/x/src/rules/mod.rs", ""));
    files.push(SourceFile::sample("crates/x/src/rules/tests.rs", ""));
    assert_eq!(
        folder_violations(&files),
        vec!["crates/x/src/rules holds 13 files, above 12"]
    );
    files.pop();
    files.pop();
    files.pop();
    assert!(folder_violations(&files).is_empty());
}
