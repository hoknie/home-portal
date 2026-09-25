use toml_edit::DocumentMut;

use super::{read_network, validate_network};
use crate::types::NetworkSettings;

fn document(text: &str) -> DocumentMut {
    text.parse().unwrap()
}

#[test]
fn a_missing_section_means_the_defaults() {
    assert_eq!(
        read_network(&document("")).unwrap(),
        NetworkSettings::default()
    );
}

#[test]
fn a_port_out_of_range_is_refused_by_its_key() {
    let errors = validate_network(&document("[network]\nport = 70000\n"));
    assert_eq!(errors[0].field, "network.port");
}

#[test]
fn every_bad_value_names_its_key() {
    let text = "[network]\naddress = \"nowhere\"\nport = 0\npublic_url = \"ftp://x\"\ntrusted_proxies = [\"10.0.0.0/8\", \"not-a-network\"]\n";
    let fields: Vec<String> = validate_network(&document(text))
        .into_iter()
        .map(|error| error.field)
        .collect();
    assert_eq!(
        fields,
        vec![
            "network.address",
            "network.port",
            "network.public_url",
            "network.trusted_proxies[1]"
        ]
    );
}

#[test]
fn a_single_address_is_a_trusted_proxy_of_its_own() {
    let settings = read_network(&document(
        "[network]\ntrusted_proxies = [\"192.168.1.1\"]\n",
    ))
    .unwrap();
    assert_eq!(settings.trusted_proxies[0].to_string(), "192.168.1.1/32");
}

#[test]
fn an_https_public_url_asks_for_secure_cookies() {
    let settings = read_network(&document(
        "[network]\npublic_url = \"https://portal.home.lan\"\n",
    ))
    .unwrap();
    assert!(settings.secure());
}

fn environments(text: &str) -> Result<portal_model::Environments, Vec<portal_feature::FieldError>> {
    crate::services::read_environments(&document(text))
}

fn environment_fields(text: &str) -> Vec<String> {
    environments(text)
        .err()
        .unwrap_or_default()
        .into_iter()
        .map(|error| error.field)
        .collect()
}

#[test]
fn a_visitor_belongs_to_the_environment_holding_their_address() {
    let text = "[environments.local]\nnetworks = [\"192.168.0.0/16\"]\n\n[environments.vpn]\nnetworks = [\"10.8.0.0/24\"]\n";
    let environments = environments(text).unwrap();
    assert_eq!(
        environments.of("192.168.1.40".parse().unwrap()).as_str(),
        "local"
    );
    assert_eq!(environments.of("10.8.0.9".parse().unwrap()).as_str(), "vpn");
    assert_eq!(
        environments.of("203.0.113.5".parse().unwrap()).as_str(),
        "internet"
    );
}

#[test]
fn without_the_section_everyone_is_on_the_internet() {
    let environments = environments("").unwrap();
    assert_eq!(
        environments.of("192.168.1.40".parse().unwrap()).as_str(),
        "internet"
    );
    assert_eq!(environments.names().len(), 1);
}

#[test]
fn the_names_list_the_configured_environments_and_the_internet() {
    let environments =
        environments("[environments.local]\nnetworks = [\"192.168.0.0/16\"]\n").unwrap();
    let names: Vec<String> = environments
        .names()
        .into_iter()
        .map(|name| name.to_string())
        .collect();
    assert_eq!(names, vec!["local", "internet"]);
}

#[test]
fn overlapping_environments_name_both() {
    let text = "[environments.local]\nnetworks = [\"10.0.0.0/8\"]\n\n[environments.vpn]\nnetworks = [\"10.8.0.0/24\"]\n";
    let errors = environments(text).err().unwrap();
    assert_eq!(errors[0].field, "environments.local.networks");
    assert!(
        errors[0].message.contains("10.8.0.0/24") && errors[0].message.contains("vpn"),
        "{}",
        errors[0].message
    );
}

#[test]
fn a_bad_name_a_bad_range_and_the_reserved_name_are_refused() {
    assert_eq!(
        environment_fields("[environments.Local]\nnetworks = [\"10.0.0.0/8\"]\n"),
        vec!["environments.Local"]
    );
    assert_eq!(
        environment_fields("[environments.local]\nnetworks = [\"nonsense\"]\n"),
        vec!["environments.local.networks[0]"]
    );
    assert_eq!(
        environment_fields("[environments.internet]\nnetworks = [\"10.0.0.0/8\"]\n"),
        vec!["environments.internet"]
    );
}

#[test]
fn an_environment_without_a_network_is_refused() {
    assert_eq!(
        environment_fields("[environments.local]\nnetworks = []\n"),
        vec!["environments.local.networks"]
    );
}
