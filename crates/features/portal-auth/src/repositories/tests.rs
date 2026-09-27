use std::fs;
use std::sync::Arc;

use portal_config::ConfigStore;
use toml_edit::DocumentMut;

use super::{append, last_origin, origin, position, remove, set_hash};

const PEOPLE: &str = "# the household\n\n[[users]]\nname = \"admin\"\npassword_hash = \"$argon2id$old\" # set by hand\n\n# my sister\n[[users]]\nname = \"anna\"\npassword_hash = \"$argon2id$anna\"\n";

fn document(text: &str) -> DocumentMut {
    text.parse().unwrap()
}

#[test]
fn a_new_user_follows_the_last_entry_and_keeps_every_comment() {
    let mut people = document(PEOPLE);
    append(&mut people, "bob", "$argon2id$bob");
    assert_eq!(
        people.to_string(),
        format!("{PEOPLE}\n[[users]]\nname = \"bob\"\npassword_hash = \"$argon2id$bob\"\n")
    );
    let mut empty = document("[network]\nport = 8080\n");
    append(&mut empty, "bob", "$argon2id$bob");
    assert!(
        empty
            .to_string()
            .ends_with("[[users]]\nname = \"bob\"\npassword_hash = \"$argon2id$bob\"\n")
    );
}

#[test]
fn a_new_hash_keeps_the_comment_beside_it() {
    let mut people = document(PEOPLE);
    let index = position(&people, "admin").unwrap();
    set_hash(&mut people, index, "$argon2id$new");
    assert_eq!(
        people.to_string(),
        PEOPLE.replace(
            "\"$argon2id$old\" # set by hand",
            "\"$argon2id$new\" # set by hand"
        )
    );
}

#[test]
fn removing_a_user_keeps_the_comments_of_the_others() {
    let mut people = document(PEOPLE);
    let index = position(&people, "admin").unwrap();
    remove(&mut people, index);
    assert_eq!(
        people.to_string(),
        "# the household\n\n# my sister\n[[users]]\nname = \"anna\"\npassword_hash = \"$argon2id$anna\"\n"
    );
    assert_eq!(position(&people, "admin"), None);
}

#[test]
fn users_are_written_back_to_the_file_they_came_from() {
    let folder = tempfile::tempdir().unwrap();
    let main = folder.path().join("home-portal.toml");
    let people = folder.path().join("people.toml");
    fs::write(&main, "include = [\"people.toml\"]\n").unwrap();
    fs::write(&people, PEOPLE).unwrap();
    let store = Arc::new(ConfigStore::open(&main).unwrap());
    let snapshot = store.read();
    assert_eq!(origin(&snapshot, "anna").as_deref(), Some(people.as_path()));
    assert_eq!(last_origin(&snapshot).as_deref(), Some(people.as_path()));
    assert_eq!(origin(&snapshot, "nobody"), None);
}
