use portal_feature::{Action, Area, Right, Rights};
use toml_edit::DocumentMut;

use super::super::groups::{NO_ADMIN, principal_for, rights_for};
use super::super::validate_users;
use crate::helpers::hash_password;
use crate::types::UsersSection;

fn with_hash(text: &str) -> String {
    text.replace("HASH", &hash_password("secret").unwrap())
}

fn errors(text: &str) -> Vec<(String, String)> {
    let document: DocumentMut = with_hash(text).parse().unwrap();
    validate_users(&document)
        .into_iter()
        .map(|error| (error.field, error.message))
        .collect()
}

fn fields(text: &str) -> Vec<String> {
    errors(text).into_iter().map(|(field, _)| field).collect()
}

const OWNER: &str = "[[users]]\nname = \"root\"\npassword_hash = \"HASH\"\ngroup = \"admin\"\n\n";

#[test]
fn a_family_group_with_a_member_passes() {
    let text = format!(
        "{OWNER}[[users]]\nname = \"anna\"\npassword_hash = \"HASH\"\ngroup = \"family\"\n\n[[groups]]\nname = \"family\"\npermissions = {{ automations = [\"read\", \"execute\"] }}\n"
    );
    assert!(errors(&text).is_empty(), "{:?}", errors(&text));
}

#[test]
fn permissions_may_be_a_sub_table() {
    let text = format!(
        "{OWNER}[[groups]]\nname = \"family\"\n\n[groups.permissions]\nservices = [\"update\"]\n"
    );
    assert!(errors(&text).is_empty(), "{:?}", errors(&text));
    let section = UsersSection::read(&with_hash(&text).parse().unwrap()).unwrap();
    assert_eq!(section.groups[0].permissions["services"], vec!["update"]);
}

#[test]
fn a_configuration_from_before_groups_names_group_admin() {
    let found = errors("[[users]]\nname = \"root\"\npassword_hash = \"HASH\"\n");
    assert_eq!(found, vec![("users".to_string(), NO_ADMIN.to_string())]);
    assert!(NO_ADMIN.contains("group = \"admin\""));
}

#[test]
fn a_group_that_does_not_exist_names_the_users_group() {
    let text = format!(
        "{OWNER}[[users]]\nname = \"anna\"\npassword_hash = \"HASH\"\ngroup = \"guests\"\n"
    );
    assert_eq!(fields(&text), vec!["users[1].group"]);
}

#[test]
fn admin_cannot_be_declared_and_names_must_be_unique() {
    let text = format!(
        "{OWNER}[[groups]]\nname = \"admin\"\n\n[[groups]]\nname = \"family\"\n\n[[groups]]\nname = \"family\"\n\n[[groups]]\nname = \" \"\n"
    );
    let found = errors(&text);
    assert_eq!(
        found
            .iter()
            .map(|(field, _)| field.as_str())
            .collect::<Vec<_>>(),
        vec!["groups[0].name", "groups[2].name", "groups[3].name"]
    );
    assert!(found[0].1.contains("built in"), "{found:?}");
}

#[test]
fn a_right_the_area_does_not_have_lists_the_accepted_actions() {
    let text = format!(
        "{OWNER}[[groups]]\nname = \"family\"\npermissions = {{ layout = [\"delete\"], cameras = [\"read\"] }}\n"
    );
    let found = errors(&text);
    assert_eq!(
        found,
        vec![
            (
                "groups[0].permissions.cameras".to_string(),
                found[0].1.clone()
            ),
            (
                "groups[0].permissions.layout".to_string(),
                "accepts only update".to_string()
            ),
        ]
    );
    assert!(found[0].1.contains("is not an area"), "{found:?}");
}

#[test]
fn rights_follow_the_group_of_the_user() {
    let text = format!(
        "{OWNER}[[users]]\nname = \"anna\"\npassword_hash = \"HASH\"\ngroup = \"family\"\n\n[[users]]\nname = \"guest\"\npassword_hash = \"HASH\"\n\n[[groups]]\nname = \"family\"\npermissions = {{ automations = [\"read\", \"execute\"] }}\n"
    );
    let section = UsersSection::read(&with_hash(&text).parse().unwrap()).unwrap();
    let rights = |name: &str| rights_for(&section, section.find(name).unwrap());
    assert_eq!(rights("root"), Rights::admin());
    assert_eq!(rights("guest"), Rights::none());
    assert_eq!(
        rights("anna"),
        Rights::of([
            Right::new(Area::Automations, Action::Read),
            Right::new(Area::Automations, Action::Execute),
        ])
    );
}

#[test]
fn the_principal_carries_the_group_and_its_rights() {
    let text =
        format!("{OWNER}[[users]]\nname = \"anna\"\npassword_hash = \"HASH\"\ngroup = \"gone\"\n");
    let section = UsersSection::read(&with_hash(&text).parse().unwrap()).unwrap();
    let root = principal_for(&section, "root");
    assert_eq!(root.group.as_deref(), Some("admin"));
    assert!(root.rights.is_admin());
    let anna = principal_for(&section, "anna");
    assert_eq!(anna.group.as_deref(), Some("gone"));
    assert_eq!(anna.rights, Rights::none());
}
