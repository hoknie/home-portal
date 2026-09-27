use portal_feature::Module;
use toml_edit::DocumentMut;

use super::{remove_legacy_switch, write_switch};

#[test]
fn a_switch_is_added_to_a_new_modules_table() {
    let mut document: DocumentMut = "[network]\nport = 8080\n".parse().unwrap();
    write_switch(&mut document, Module::Proxy, true);
    assert_eq!(
        document.to_string(),
        "[network]\nport = 8080\n\n[modules]\nproxy = true\n"
    );
}

#[test]
fn an_existing_switch_keeps_its_comment() {
    let mut document: DocumentMut = "# parts\n[modules]\ndns = true # for the router\n"
        .parse()
        .unwrap();
    write_switch(&mut document, Module::Dns, false);
    assert_eq!(
        document.to_string(),
        "# parts\n[modules]\ndns = false # for the router\n"
    );
}

#[test]
fn a_legacy_switch_is_removed_and_the_rest_of_its_section_kept() {
    let mut document: DocumentMut = "# names\n[dns]\nenabled = true # on\nport = 53\n"
        .parse()
        .unwrap();
    remove_legacy_switch(&mut document, Module::Dns);
    assert_eq!(document.to_string(), "# names\n[dns]\nport = 53\n");
    let mut untouched: DocumentMut = "[proxy]\nportal_host = \"p.example.com\"\n"
        .parse()
        .unwrap();
    remove_legacy_switch(&mut untouched, Module::Automations);
    remove_legacy_switch(&mut untouched, Module::Proxy);
    assert_eq!(
        untouched.to_string(),
        "[proxy]\nportal_host = \"p.example.com\"\n"
    );
}
