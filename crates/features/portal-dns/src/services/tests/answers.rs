use portal_model::Environment;

use super::support::{book, environment, published};
use crate::services::answer;
use crate::types::{Question, RecordData, RecordKind, ReplyCode, ZoneBook};

fn split() -> ZoneBook {
    book(
        "[dns]\nzones = [\"home\"]\nttl = 120\n[dns.addresses]\nlocal = \"192.168.1.60\"\nvpn = \"10.8.0.1\"\n[[dns.records]]\nname = \"media.home\"\ntype = \"CNAME\"\nvalue = \"jellyfin.home\"\n",
        &[
            published("jellyfin.home", None),
            published("nas.home", Some(&["local"])),
        ],
        &[],
    )
}

fn ask(book: &ZoneBook, from: &Environment, name: &str, kind: RecordKind) -> crate::types::Reply {
    answer(
        book,
        from,
        &Question {
            name: name.into(),
            kind,
        },
    )
}

#[test]
fn split_horizon_answers_the_proxy_address_of_the_asking_network() {
    let book = split();
    let home = ask(&book, &environment("local"), "jellyfin.home", RecordKind::A);
    let away = ask(&book, &environment("vpn"), "jellyfin.home", RecordKind::A);
    assert_eq!(home.answers[0].data.text(), "192.168.1.60");
    assert_eq!(away.answers[0].data.text(), "10.8.0.1");
    assert_eq!(home.answers[0].ttl, 120);
    assert!(home.authoritative);
}

#[test]
fn a_service_kept_at_home_is_a_name_error_with_the_zone_authority_over_the_vpn() {
    let reply = ask(&split(), &environment("vpn"), "nas.home", RecordKind::A);
    assert_eq!(reply.code, ReplyCode::NameError);
    assert!(matches!(reply.authority[0].data, RecordData::Soa { .. }));
    assert_eq!(reply.authority[0].name, "home");
}

#[test]
fn a_foreign_name_is_refused() {
    let reply = ask(
        &split(),
        &environment("local"),
        "example.org",
        RecordKind::A,
    );
    assert_eq!(reply.code, ReplyCode::Refused);
    assert!(reply.answers.is_empty() && reply.authority.is_empty());
}

#[test]
fn a_known_name_without_the_asked_type_answers_nothing_but_the_authority() {
    let reply = ask(
        &split(),
        &environment("local"),
        "jellyfin.home",
        RecordKind::Aaaa,
    );
    assert_eq!(reply.code, ReplyCode::NoError);
    assert!(reply.answers.is_empty());
    assert_eq!(reply.authority.len(), 1);
}

#[test]
fn the_apex_answers_its_soa_and_ns() {
    let book = split();
    let soa = ask(&book, &environment("local"), "home", RecordKind::Soa);
    assert!(
        matches!(&soa.answers[0].data, RecordData::Soa { serial, .. } if *serial == book.serial)
    );
    let ns = ask(&book, &environment("local"), "home", RecordKind::Ns);
    assert_eq!(ns.answers[0].data.text(), "portal.home");
}

#[test]
fn an_alias_is_followed_once() {
    let reply = ask(&split(), &environment("local"), "media.home", RecordKind::A);
    assert_eq!(reply.answers.len(), 2);
    assert_eq!(reply.answers[0].data.text(), "jellyfin.home");
    assert_eq!(reply.answers[1].data.text(), "192.168.1.60");
}

#[test]
fn any_and_transfers_are_refused() {
    assert_eq!(
        ask(&split(), &environment("local"), "home", RecordKind::Any).code,
        ReplyCode::Refused
    );
    assert_eq!(
        ask(
            &split(),
            &environment("local"),
            "home",
            RecordKind::Transfer
        )
        .code,
        ReplyCode::Refused
    );
}

#[test]
fn a_question_from_the_internet_is_refused_with_empty_sections() {
    let reply = ask(
        &split(),
        &Environment::internet(),
        "jellyfin.home",
        RecordKind::A,
    );
    assert_eq!(reply.code, ReplyCode::Refused);
    assert!(reply.answers.is_empty() && reply.authority.is_empty());
}
