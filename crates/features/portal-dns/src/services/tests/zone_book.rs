use std::net::Ipv4Addr;

use super::support::{book, environment, published};
use crate::types::RecordData;

const ZONES: &str = "[dns]\nzones = [\"home\"]\n";

#[test]
fn the_proxy_address_is_taken_from_the_interface_inside_the_environment() {
    let found = book(
        ZONES,
        &[published("jellyfin.home", None)],
        &["172.16.0.2", "192.168.1.60", "fd00::60"],
    );
    assert_eq!(
        found.records("jellyfin.home", &environment("local")),
        Some(
            &[
                RecordData::A(Ipv4Addr::new(192, 168, 1, 60)),
                RecordData::Aaaa("fd00::60".parse().unwrap())
            ][..]
        )
    );
}

#[test]
fn an_environment_without_an_address_is_reported_and_has_no_answers() {
    let found = book(
        ZONES,
        &[published("jellyfin.home", None)],
        &["192.168.1.60"],
    );
    assert_eq!(found.unaddressed, vec![environment("vpn")]);
    assert!(
        found
            .records("jellyfin.home", &environment("vpn"))
            .is_none()
    );
}

#[test]
fn a_configured_address_wins_over_the_interfaces() {
    let found = book(
        "[dns]\nzones = [\"home\"]\n[dns.addresses]\nvpn = \"10.8.0.1\"\n",
        &[published("jellyfin.home", None)],
        &["192.168.1.60"],
    );
    assert_eq!(
        found.records("jellyfin.home", &environment("vpn")),
        Some(&[RecordData::A(Ipv4Addr::new(10, 8, 0, 1))][..])
    );
    assert!(found.unaddressed.is_empty());
}

#[test]
fn a_host_outside_the_zones_is_its_own_single_name_zone() {
    let found = book(
        ZONES,
        &[published("photos.example.com", None)],
        &["192.168.1.60"],
    );
    let zone = found.zone_of("photos.example.com").unwrap();
    assert!(zone.single);
    assert!(found.zone_of("other.example.com").is_none());
}

#[test]
fn a_service_shown_only_at_home_has_no_answer_over_the_vpn() {
    let found = book(
        "[dns]\nzones = [\"home\"]\n[dns.addresses]\nvpn = \"10.8.0.1\"\n",
        &[published("nas.home", Some(&["local"]))],
        &["192.168.1.60"],
    );
    assert!(found.records("nas.home", &environment("local")).is_some());
    assert!(found.records("nas.home", &environment("vpn")).is_none());
}

#[test]
fn the_serial_changes_with_the_answers_and_stays_otherwise() {
    let first = book(
        ZONES,
        &[published("jellyfin.home", None)],
        &["192.168.1.60"],
    );
    let same = book(
        ZONES,
        &[published("jellyfin.home", None)],
        &["192.168.1.60"],
    );
    let changed = book(
        ZONES,
        &[
            published("jellyfin.home", None),
            published("photos.home", None),
        ],
        &["192.168.1.60"],
    );
    assert_eq!(first.serial, same.serial);
    assert_ne!(first.serial, changed.serial);
}

#[test]
fn the_owners_records_answer_in_every_environment_when_they_name_none() {
    let found = book(
        "[dns]\nzones = [\"home\"]\n[[dns.records]]\nname = \"printer.home\"\ntype = \"A\"\nvalue = \"192.168.1.9\"\n",
        &[],
        &["192.168.1.60"],
    );
    assert!(
        found
            .records("portal.home", &environment("local"))
            .is_some()
    );
    assert!(found.records("printer.home", &environment("vpn")).is_some());
}
