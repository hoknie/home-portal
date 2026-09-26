use std::net::IpAddr;

use hickory_proto::op::{Message, MessageType, OpCode, Query, ResponseCode};
use hickory_proto::rr::{Name, RecordType};

use super::support::{book, published};
use crate::services::{UDP_LIMIT, handle};

fn query(name: &str, record_type: RecordType) -> Vec<u8> {
    let mut message = Message::new();
    message
        .set_id(7)
        .add_query(Query::query(Name::from_ascii(name).unwrap(), record_type));
    message.to_vec().unwrap()
}

fn peer(text: &str) -> IpAddr {
    text.parse().unwrap()
}

#[test]
fn an_ipv4_peer_written_as_ipv6_is_answered_as_home() {
    let book = book(
        "[dns]\nzones = [\"home\"]\n",
        &[published("jellyfin.home", None)],
        &["192.168.1.60"],
    );
    let bytes = handle(
        &book,
        peer("::ffff:192.168.1.40"),
        &query("jellyfin.home.", RecordType::A),
        Some(UDP_LIMIT),
    )
    .unwrap();
    let message = Message::from_vec(&bytes).unwrap();
    assert_eq!(message.response_code(), ResponseCode::NoError);
    assert_eq!(message.answers()[0].data().to_string(), "192.168.1.60");
}

#[test]
fn an_oversized_answer_over_udp_comes_back_truncated() {
    let records: String = (0..40)
        .map(|index| {
            format!(
                "[[dns.records]]\nname = \"big.home\"\ntype = \"TXT\"\nvalue = \"{index:0>60}\"\n"
            )
        })
        .collect();
    let book = book(&format!("[dns]\nzones = [\"home\"]\n{records}"), &[], &[]);
    let bytes = handle(
        &book,
        peer("192.168.1.40"),
        &query("big.home.", RecordType::TXT),
        Some(UDP_LIMIT),
    )
    .unwrap();
    assert!(bytes.len() <= UDP_LIMIT);
    assert!(Message::from_vec(&bytes).unwrap().truncated());
    let whole = handle(
        &book,
        peer("192.168.1.40"),
        &query("big.home.", RecordType::TXT),
        None,
    )
    .unwrap();
    assert_eq!(Message::from_vec(&whole).unwrap().answers().len(), 40);
}

#[test]
fn the_internet_a_notify_and_garbage_are_told_off() {
    let book = book("[dns]\nzones = [\"home\"]\n", &[], &[]);
    let refused = handle(
        &book,
        peer("203.0.113.7"),
        &query("portal.home.", RecordType::A),
        Some(UDP_LIMIT),
    )
    .unwrap();
    let message = Message::from_vec(&refused).unwrap();
    assert_eq!(message.response_code(), ResponseCode::Refused);
    assert!(message.answers().is_empty() && message.name_servers().is_empty());
    let mut notify = Message::new();
    notify.set_op_code(OpCode::Notify).add_query(Query::query(
        Name::from_ascii("home.").unwrap(),
        RecordType::SOA,
    ));
    let answered = handle(&book, peer("192.168.1.40"), &notify.to_vec().unwrap(), None).unwrap();
    let answered = Message::from_vec(&answered).unwrap();
    assert_eq!(answered.response_code(), ResponseCode::NotImp);
    assert_eq!(answered.message_type(), MessageType::Response);
    assert!(handle(&book, peer("192.168.1.40"), b"garbage", None).is_none());
}
