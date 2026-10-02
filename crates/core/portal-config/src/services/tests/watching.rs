use std::fs;

use crate::services::configuration_files;

#[test]
fn the_watched_files_are_the_main_file_every_section_home_and_the_workflows() {
    let directory = tempfile::tempdir().unwrap();
    let main = directory.path().join("home-portal.toml");
    fs::write(&main, "").unwrap();
    fs::create_dir(directory.path().join("workflows")).unwrap();
    fs::write(
        directory.path().join("workflows/revive.toml"),
        "id = \"revive\"\n",
    )
    .unwrap();
    let files = configuration_files(&main);
    assert_eq!(files[0], main);
    for name in ["users.toml", "services.toml", "proxy.toml", "secrets.toml"] {
        assert!(
            files.contains(&directory.path().join(name)),
            "{name}: {files:?}"
        );
    }
    assert!(files.contains(&directory.path().join("workflows/revive.toml")));
    assert!(files.contains(&directory.path().join("workflows")));
}

#[test]
fn a_moved_part_is_watched_where_the_main_file_puts_it() {
    let directory = tempfile::tempdir().unwrap();
    let main = directory.path().join("home-portal.toml");
    fs::write(&main, "[files]\nautomations = \"rules.toml\"\n").unwrap();
    assert!(configuration_files(&main).contains(&directory.path().join("rules.toml")));
}

#[test]
fn a_main_file_that_does_not_parse_still_gives_the_default_homes() {
    let directory = tempfile::tempdir().unwrap();
    let main = directory.path().join("home-portal.toml");
    fs::write(&main, "[network\n").unwrap();
    assert!(configuration_files(&main).contains(&directory.path().join("users.toml")));
}
