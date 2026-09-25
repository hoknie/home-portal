use crate::sources::directories;

pub const FORBIDDEN: [&str; 8] = [
    "utils", "util", "common", "misc", "structs", "enums", "traits", "impls",
];

pub fn violations(directories: &[String]) -> Vec<String> {
    directories
        .iter()
        .filter(|directory| directory.contains("/src/"))
        .filter(|directory| {
            let name = directory.rsplit('/').next().unwrap_or(directory);
            FORBIDDEN.contains(&name)
        })
        .map(|directory| format!("{directory} is a catch-all folder"))
        .collect()
}

#[test]
fn no_folder_under_src_has_a_catch_all_name() {
    let found = violations(&directories());
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_utils_folder_is_caught_by_path() {
    let sample = vec![
        "crates/features/portal-health/src".to_string(),
        "crates/features/portal-health/src/utils".to_string(),
        "crates/features/portal-health/src/helpers".to_string(),
    ];
    assert_eq!(
        violations(&sample),
        vec!["crates/features/portal-health/src/utils is a catch-all folder"]
    );
}
