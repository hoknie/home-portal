use crate::scanner::scan;
use crate::sources::{SourceFile, rust_files};

pub const CONTROLLERS: &str = "/src/controllers/";
pub const ROOT_ENTRY_POINTS: [&str; 3] = [
    "bin/home-portal/src/adapters/",
    "bin/home-portal/src/middlewares/",
    "bin/home-portal/src/cli/",
];
pub const COMMAND_LINE: &str = "bin/home-portal/src/cli/";
pub const FORBIDDEN_NAMES: [&str; 3] = ["repositories", "ConfigStore", "Snapshot"];
pub const FORBIDDEN_FIELDS: [&str; 2] = ["document", "configuration"];
pub const STORE: &str = "ConfigStore";
pub const TESTS_FILE: &str = "tests.rs";
pub const TESTS_FOLDER: &str = "/tests/";
pub const CRATE_ROOT: &str = "lib.rs";
pub const REPOSITORY_EXPORTS: [&str; 2] = ["pubuserepositories", "pubusecrate::repositories"];

pub fn violations(files: &[SourceFile]) -> Vec<String> {
    let mut found = Vec::new();
    for file in files {
        if is_entry_point(file) {
            entry_point_violations(file, &mut found);
        }
        if file.path.starts_with("crates/") && file.file_name() == CRATE_ROOT {
            export_violations(file, &mut found);
        }
    }
    found
}

pub fn is_entry_point(file: &SourceFile) -> bool {
    let placed = file.path.contains(CONTROLLERS)
        || ROOT_ENTRY_POINTS
            .iter()
            .any(|prefix| file.path.starts_with(prefix));
    let source = file
        .path
        .split_once("/src/")
        .map_or(file.path.as_str(), |(_, inside)| inside);
    placed && file.file_name() != TESTS_FILE && !format!("/{source}").contains(TESTS_FOLDER)
}

fn entry_point_violations(file: &SourceFile, found: &mut Vec<String>) {
    let may_open_store = file.path.starts_with(COMMAND_LINE);
    let code = scan(&file.text).code;
    for (index, line) in code.lines().enumerate() {
        for name in FORBIDDEN_NAMES {
            let permitted = may_open_store && name == STORE;
            if !permitted && words(line).any(|word| word == name) {
                found.push(format!("{}:{} names {name}", file.path, index + 1));
            }
        }
        for field in FORBIDDEN_FIELDS {
            if accesses(line, field) {
                found.push(format!("{}:{} names .{field}", file.path, index + 1));
            }
        }
    }
}

fn export_violations(file: &SourceFile, found: &mut Vec<String>) {
    let code = scan(&file.text).code;
    for line in code.lines() {
        let compact: String = line
            .chars()
            .filter(|current| !current.is_whitespace())
            .collect();
        if REPOSITORY_EXPORTS
            .iter()
            .any(|export| compact.starts_with(export))
        {
            found.push(format!("{} re-exports repositories", file.path));
        }
    }
}

fn words(line: &str) -> impl Iterator<Item = &str> {
    line.split(|current: char| !(current.is_alphanumeric() || current == '_'))
}

fn accesses(line: &str, field: &str) -> bool {
    line.match_indices('.').any(|(position, _)| {
        let rest = &line[position + 1..];
        rest.starts_with(field)
            && !rest[field.len()..]
                .chars()
                .next()
                .is_some_and(|next| next.is_alphanumeric() || next == '_')
    })
}

#[test]
fn entry_points_reach_storage_only_through_use_cases() {
    let found = violations(&rust_files());
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_controller_importing_its_repository_is_caught() {
    let controller = SourceFile::sample(
        "crates/features/portal-network/src/controllers/network.rs",
        "use crate::repositories::write_network;\n",
    );
    assert_eq!(
        violations(&[controller]),
        vec!["crates/features/portal-network/src/controllers/network.rs:1 names repositories"]
    );
}

#[test]
fn a_controller_writing_through_its_state_is_caught() {
    let controller = SourceFile::sample(
        "crates/features/portal-dns/src/controllers/dns.rs",
        "fn a() {\n    state\n        .configuration\n        .update(target);\n}\n",
    );
    assert_eq!(
        violations(&[controller]),
        vec!["crates/features/portal-dns/src/controllers/dns.rs:3 names .configuration"]
    );
}

#[test]
fn a_middleware_reading_the_document_is_caught() {
    let middleware = SourceFile::sample(
        "bin/home-portal/src/middlewares/environment.rs",
        "fn a() {\n    let settings = read(&store.read().document);\n}\n",
    );
    assert_eq!(
        violations(&[middleware]),
        vec!["bin/home-portal/src/middlewares/environment.rs:2 names .document"]
    );
}

#[test]
fn an_adapter_calling_a_use_case_passes() {
    let adapter = SourceFile::sample(
        "bin/home-portal/src/adapters/dns_directory.rs",
        "fn a(&self) -> Environments {\n    self.environments.run()\n}\n",
    );
    assert!(violations(&[adapter]).is_empty());
}

#[test]
fn the_command_line_may_open_the_store_but_not_read_it() {
    let opening = SourceFile::sample(
        "bin/home-portal/src/cli/probe.rs",
        "use portal_config::ConfigStore;\nfn a() { ConfigStore::open_located(&location); }\n",
    );
    let reading = SourceFile::sample(
        "bin/home-portal/src/cli/proxy_render.rs",
        "fn a() { let document = store.read().document; }\n",
    );
    assert_eq!(
        violations(&[opening, reading]),
        vec!["bin/home-portal/src/cli/proxy_render.rs:1 names .document"]
    );
}

#[test]
fn a_repository_leaking_through_the_crate_root_is_caught() {
    let root = SourceFile::sample(
        "crates/features/portal-auth/src/lib.rs",
        "mod repositories;\n\npub use repositories::SessionFile;\n",
    );
    assert_eq!(
        violations(&[root]),
        vec!["crates/features/portal-auth/src/lib.rs re-exports repositories"]
    );
}

#[test]
fn controller_tests_may_build_a_store() {
    let beside = SourceFile::sample(
        "crates/features/portal-services/src/controllers/tests.rs",
        "use portal_config::ConfigStore;\n",
    );
    let folder = SourceFile::sample(
        "crates/features/portal-proxy/src/controllers/tests/support.rs",
        "use portal_config::ConfigStore;\nfn a() { state.configuration.read().document; }\n",
    );
    assert!(violations(&[beside, folder]).is_empty());
}

#[test]
fn names_inside_strings_and_longer_words_are_not_caught() {
    let controller = SourceFile::sample(
        "crates/features/portal-dns/src/controllers/dns.rs",
        "const A: &str = \"configuration.document\";\nfn a() { state.documents; DnsSnapshot::new(); }\n",
    );
    assert!(violations(&[controller]).is_empty());
}
