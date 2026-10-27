use super::entry_points::{TESTS_FILE, TESTS_FOLDER, words};
use crate::scanner::scan;
use crate::sources::{SourceFile, rust_files};

pub const REPOSITORIES: &str = "/src/repositories/";
pub const TESTS_SUFFIX: &str = "_tests.rs";
pub const UPPER_KINDS: [&str; 6] = [
    "services",
    "usecases",
    "loops",
    "controllers",
    "requests",
    "responses",
];
pub const IMPORT_ROOTS: [&str; 2] = ["crate::", "super::super::"];
pub const CLOCK_OWNERS: [&str; 3] = ["OffsetDateTime", "SystemTime", "Instant"];
pub const CLOCK_WORD: &str = "now_utc";

pub fn is_repository(file: &SourceFile) -> bool {
    let Some((_, inside)) = file.path.split_once(REPOSITORIES) else {
        return false;
    };
    let name = file.file_name();
    name != TESTS_FILE
        && !name.ends_with(TESTS_SUFFIX)
        && !format!("/{inside}").contains(TESTS_FOLDER)
}

pub fn repository_violations(files: &[SourceFile]) -> Vec<String> {
    let mut found = Vec::new();
    for file in files.iter().filter(|file| is_repository(file)) {
        let code = scan(&file.text).code;
        for (line, statement) in use_statements(&code) {
            for kind in imported_kinds(&statement) {
                found.push(format!("{}:{line} imports {kind}", file.path));
            }
        }
        for (index, line) in code.lines().enumerate() {
            if line.trim_start().starts_with("use ") || line.trim_start().starts_with("pub use ") {
                continue;
            }
            for kind in path_kinds(line) {
                found.push(format!("{}:{} imports {kind}", file.path, index + 1));
            }
            for name in clock_reads(line) {
                found.push(format!(
                    "{}:{} reads the clock with {name}",
                    file.path,
                    index + 1
                ));
            }
        }
    }
    found
}

fn use_statements(code: &str) -> Vec<(usize, String)> {
    let mut statements = Vec::new();
    let mut current: Option<(usize, String)> = None;
    for (index, line) in code.lines().enumerate() {
        let trimmed = line.trim_start();
        if current.is_none() && (trimmed.starts_with("use ") || trimmed.starts_with("pub use ")) {
            current = Some((index + 1, String::new()));
        }
        if let Some((_, text)) = current.as_mut() {
            text.push_str(line);
            if line.contains(';') {
                statements.extend(current.take());
            }
        }
    }
    statements
}

fn imported_kinds(statement: &str) -> Vec<&'static str> {
    let compact: String = statement
        .chars()
        .filter(|current| !current.is_whitespace())
        .collect();
    let path = compact
        .trim_start_matches("pub")
        .trim_start_matches("use")
        .trim_end_matches(';');
    let Some(rest) = IMPORT_ROOTS.iter().find_map(|root| path.strip_prefix(root)) else {
        return Vec::new();
    };
    let heads: Vec<&str> = match rest.strip_prefix('{') {
        Some(group) => top_level_items(group.trim_end_matches('}')),
        None => vec![rest],
    };
    heads
        .into_iter()
        .filter_map(|item| {
            let head = item.split("::").next().unwrap_or_default();
            UPPER_KINDS.iter().copied().find(|kind| *kind == head)
        })
        .collect()
}

fn top_level_items(group: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let (mut depth, mut start) = (0usize, 0usize);
    for (position, current) in group.char_indices() {
        match current {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                items.push(&group[start..position]);
                start = position + 1;
            }
            _ => {}
        }
    }
    items.push(&group[start..]);
    items
}

fn path_kinds(line: &str) -> Vec<&'static str> {
    let compact: String = line
        .chars()
        .filter(|current| !current.is_whitespace())
        .collect();
    UPPER_KINDS
        .iter()
        .copied()
        .filter(|kind| {
            IMPORT_ROOTS
                .iter()
                .any(|root| compact.contains(&format!("{root}{kind}::")))
        })
        .collect()
}

fn clock_reads(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    if words(line).any(|word| word == CLOCK_WORD) {
        found.push(CLOCK_WORD.to_string());
    }
    let compact: String = line
        .chars()
        .filter(|current| !current.is_whitespace())
        .collect();
    for owner in CLOCK_OWNERS {
        let call = format!("{owner}::now");
        let named = compact.match_indices(&call).any(|(position, _)| {
            !compact[position + call.len()..]
                .chars()
                .next()
                .is_some_and(|next| next.is_alphanumeric() || next == '_')
        });
        if named {
            found.push(call);
        }
    }
    found
}

#[test]
fn repositories_only_access_storage() {
    let found = repository_violations(&rust_files());
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_repository_importing_a_wire_type_or_a_service_is_caught() {
    let wire = SourceFile::sample(
        "crates/features/portal-dashboard/src/repositories/layout.rs",
        "use crate::responses::WidgetView;\n",
    );
    let service = SourceFile::sample(
        "crates/features/portal-services/src/repositories/history.rs",
        "use std::fs;\n\nuse crate::services::ServiceHistory;\n",
    );
    assert_eq!(
        repository_violations(&[wire, service]),
        vec![
            "crates/features/portal-dashboard/src/repositories/layout.rs:1 imports responses",
            "crates/features/portal-services/src/repositories/history.rs:3 imports services",
        ]
    );
}

#[test]
fn a_grouped_or_full_path_import_of_an_upper_layer_is_caught() {
    let grouped = SourceFile::sample(
        "crates/features/portal-proxy/src/repositories/proxy_section.rs",
        "use crate::{\n    services::Settings,\n    types::ProxyChoice,\n};\nfn a() { super::super::usecases::PrepareProxy; }\n",
    );
    assert_eq!(
        repository_violations(&[grouped]),
        vec![
            "crates/features/portal-proxy/src/repositories/proxy_section.rs:1 imports services",
            "crates/features/portal-proxy/src/repositories/proxy_section.rs:5 imports usecases",
        ]
    );
}

#[test]
fn a_repository_reading_the_clock_is_caught() {
    let clock = SourceFile::sample(
        "crates/features/portal-services/src/repositories/history.rs",
        "fn a() {\n    let now = OffsetDateTime::now_utc();\n    let started = Instant::now();\n}\n",
    );
    assert_eq!(
        repository_violations(&[clock]),
        vec![
            "crates/features/portal-services/src/repositories/history.rs:2 reads the clock with now_utc",
            "crates/features/portal-services/src/repositories/history.rs:3 reads the clock with Instant::now",
        ]
    );
}

#[test]
fn a_repository_may_use_domain_types_and_tests_may_use_anything() {
    let domain = SourceFile::sample(
        "crates/features/portal-services/src/repositories/services.rs",
        "use portal_model::{Publication, TlsPolicy};\n\nuse crate::types::{ProbeSettings, ServiceEntry};\nconst A: &str = \"crate::services::X now_utc\";\n",
    );
    let tests = [
        "crates/features/portal-automations/src/repositories/tests.rs",
        "crates/features/portal-automations/src/repositories/workflow_tests.rs",
        "crates/features/portal-services/src/repositories/tests/history.rs",
    ]
    .map(|path| {
        SourceFile::sample(
            path,
            "use crate::services::Journal;\nfn a() { OffsetDateTime::now_utc(); }\n",
        )
    });
    assert!(repository_violations(&[domain]).is_empty());
    assert!(repository_violations(&tests).is_empty());
}
