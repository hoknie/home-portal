use std::net::{IpAddr, Ipv4Addr};

use toml_edit::DocumentMut;

use super::NetworkOfDocument;

fn document(text: &str) -> DocumentMut {
    text.parse().unwrap()
}

#[test]
fn a_lone_main_file_gives_its_address_and_trusted_proxies() {
    let reading = NetworkOfDocument.run(&document(
        "[network]\naddress = \"0.0.0.0\"\nport = 9191\ntrusted_proxies = [\"127.0.0.1/32\"]\n",
    ));
    let settings = reading.settings.unwrap();
    assert_eq!(settings.socket_address().to_string(), "0.0.0.0:9191");
    assert_eq!(settings.trusted_proxies.len(), 1);
}

#[test]
fn a_lone_main_file_gives_its_environments() {
    let reading = NetworkOfDocument.run(&document(
        "[environments.local]\nnetworks = [\"192.168.1.0/24\"]\n",
    ));
    let environments = reading.environments.unwrap();
    let phone = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 40));
    assert_eq!(environments.of(phone).as_str(), "local");
}

#[test]
fn a_broken_network_section_keeps_the_environments() {
    let reading = NetworkOfDocument.run(&document(
        "[network]\nport = 70000\n\n[environments.local]\nnetworks = [\"192.168.1.0/24\"]\n",
    ));
    assert!(reading.settings.is_err());
    assert!(reading.environments.is_ok());
}
