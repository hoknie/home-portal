use std::fs;

use super::{opened, split};

#[test]
fn an_all_in_one_file_is_split_into_the_homes_and_opens() {
    let directory = tempfile::tempdir().unwrap();
    let main = directory.path().join("home-portal.toml");
    fs::write(
        &main,
        "[network]\nport = 8080\n\n[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://nas.lan\"\n\n[secrets]\ntoken = \"x\"\n\n[[workflows]]\nid = \"revive\"\ntitle = \"Revive\"\n\n[[workflows.steps]]\nid = \"pause\"\nkind = \"wait\"\nseconds = 1\n",
    )
    .unwrap();
    let store = opened(&main).unwrap();
    assert!(
        fs::read_to_string(directory.path().join("services.toml"))
            .unwrap()
            .contains("[[services]]")
    );
    assert!(
        fs::read_to_string(directory.path().join("workflows/revive.toml"))
            .unwrap()
            .contains("[[steps]]")
    );
    assert!(!fs::read_to_string(&main).unwrap().contains("services"));
    assert_eq!(
        store
            .secret("token")
            .map(|secret| secret.expose().to_string()),
        Some("x".to_string())
    );
}

#[test]
fn a_home_named_in_files_is_honoured_and_an_existing_home_is_appended_to() {
    let directory = tempfile::tempdir().unwrap();
    let main = directory.path().join("home-portal.toml");
    fs::write(
        directory.path().join("people.toml"),
        "[[users]]\nname = \"anna\"\n",
    )
    .unwrap();
    fs::write(
        &main,
        "[files]\nusers = \"people.toml\"\n\n[[users]]\nname = \"bob\"\n",
    )
    .unwrap();
    split(&main);
    let people = fs::read_to_string(directory.path().join("people.toml")).unwrap();
    assert!(
        people.contains("anna") && people.contains("bob"),
        "{people}"
    );
}
