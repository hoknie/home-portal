use std::net::Ipv4Addr;

use hickory_proto::op::{Message, MessageType, Query};
use hickory_proto::rr::{Name, RecordType};

use super::{read, write};
use crate::types::{RecordData, RecordKind, Reply, ReplyCode, ResourceRecord};

pub fn query_bytes(name: &str, record_type: RecordType) -> Vec<u8> {
    let mut message = Message::new();
    message
        .set_id(4242)
        .set_recursion_desired(true)
        .add_query(Query::query(Name::from_ascii(name).unwrap(), record_type));
    message.to_vec().unwrap()
}

#[test]
fn a_query_is_read_with_its_name_lower_cased_and_its_kind() {
    let incoming = read(&query_bytes("Jellyfin.HOME.", RecordType::A)).unwrap();
    assert_eq!(incoming.id, 4242);
    assert!(incoming.query);
    assert!(incoming.recursion_desired);
    let question = incoming.question.unwrap();
    assert_eq!(question.name, "jellyfin.home");
    assert_eq!(question.kind, RecordKind::A);
}

#[test]
fn an_answer_reads_back_with_the_question_and_its_records() {
    let incoming = read(&query_bytes("jellyfin.home.", RecordType::A)).unwrap();
    let reply = Reply {
        code: ReplyCode::NoError,
        authoritative: true,
        answers: vec![ResourceRecord {
            name: "jellyfin.home".into(),
            ttl: 60,
            data: RecordData::A(Ipv4Addr::new(192, 168, 1, 60)),
        }],
        authority: Vec::new(),
    };
    let bytes = write(&incoming, &reply, None).unwrap();
    let message = Message::from_vec(&bytes).unwrap();
    assert_eq!(message.id(), 4242);
    assert_eq!(message.message_type(), MessageType::Response);
    assert!(message.authoritative());
    assert!(!message.recursion_available());
    assert_eq!(message.queries()[0].name().to_ascii(), "jellyfin.home.");
    assert_eq!(message.answers()[0].data().to_string(), "192.168.1.60");
    assert_eq!(message.answers()[0].ttl(), 60);
}

#[test]
fn two_questions_a_response_and_a_broken_message_are_told_apart() {
    let mut two = Message::new();
    two.add_query(Query::query(
        Name::from_ascii("a.home.").unwrap(),
        RecordType::A,
    ));
    two.add_query(Query::query(
        Name::from_ascii("b.home.").unwrap(),
        RecordType::A,
    ));
    assert!(read(&two.to_vec().unwrap()).unwrap().question.is_none());
    let mut response = Message::new();
    response.set_message_type(MessageType::Response);
    response.add_query(Query::query(
        Name::from_ascii("a.home.").unwrap(),
        RecordType::A,
    ));
    assert!(!read(&response.to_vec().unwrap()).unwrap().query);
    let bytes = query_bytes("a.home.", RecordType::A);
    assert!(read(&bytes[..bytes.len() - 3]).is_none());
    assert_eq!(
        read(&query_bytes("home.", RecordType::AXFR))
            .unwrap()
            .question
            .unwrap()
            .kind,
        RecordKind::Transfer
    );
}

#[test]
fn an_answer_above_the_limit_is_truncated_to_its_question() {
    let incoming = read(&query_bytes("big.home.", RecordType::TXT)).unwrap();
    let reply = Reply {
        code: ReplyCode::NoError,
        authoritative: true,
        answers: (0..40)
            .map(|index| ResourceRecord {
                name: "big.home".into(),
                ttl: 60,
                data: RecordData::Txt(format!("{index:0>60}")),
            })
            .collect(),
        authority: Vec::new(),
    };
    let bytes = write(&incoming, &reply, Some(1232)).unwrap();
    assert!(bytes.len() <= 1232);
    let message = Message::from_vec(&bytes).unwrap();
    assert!(message.truncated());
    assert!(message.answers().is_empty());
    assert_eq!(message.queries().len(), 1);
}
