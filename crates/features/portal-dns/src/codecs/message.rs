use hickory_proto::op::{Message, MessageType, OpCode, Query, ResponseCode};
use hickory_proto::rr::rdata::{A, AAAA, CNAME, NS, SOA, TXT};
use hickory_proto::rr::{Name, RData, Record, RecordType};

use crate::types::{Incoming, Question, RecordData, RecordKind, Reply, ReplyCode, ResourceRecord};

pub const REFRESH_SECONDS: i32 = 3600;
pub const RETRY_SECONDS: i32 = 600;
pub const EXPIRE_SECONDS: i32 = 86400;

pub fn read(bytes: &[u8]) -> Option<Incoming> {
    let message = Message::from_vec(bytes).ok()?;
    let query = message.message_type() == MessageType::Query && message.op_code() == OpCode::Query;
    let single = message.queries().len() == 1;
    let first = message.queries().first();
    Some(Incoming {
        id: message.id(),
        recursion_desired: message.recursion_desired(),
        query,
        question: first.filter(|_| single).map(|query| Question {
            name: plain(query.name()),
            kind: kind_of(query.query_type()),
        }),
        question_type: first.map_or(0, |query| u16::from(query.query_type())),
    })
}

pub fn write(incoming: &Incoming, reply: &Reply, limit: Option<usize>) -> Option<Vec<u8>> {
    let full = message(incoming, reply, true).to_vec().ok()?;
    match limit {
        Some(limit) if full.len() > limit => {
            let mut truncated = message(incoming, reply, false);
            truncated.set_truncated(true);
            truncated.to_vec().ok()
        }
        _ => Some(full),
    }
}

fn message(incoming: &Incoming, reply: &Reply, with_records: bool) -> Message {
    let mut message = Message::new();
    message
        .set_id(incoming.id)
        .set_message_type(MessageType::Response)
        .set_op_code(OpCode::Query)
        .set_authoritative(reply.authoritative)
        .set_recursion_desired(incoming.recursion_desired)
        .set_recursion_available(false)
        .set_response_code(code_of(reply.code));
    if let Some(question) = &incoming.question
        && let Ok(name) = fqdn(&question.name)
    {
        message.add_query(Query::query(name, RecordType::from(incoming.question_type)));
    }
    if with_records {
        for record in reply.answers.iter().filter_map(record_of) {
            message.add_answer(record);
        }
        for record in reply.authority.iter().filter_map(record_of) {
            message.add_name_server(record);
        }
    }
    message
}

fn record_of(record: &ResourceRecord) -> Option<Record> {
    let name = fqdn(&record.name).ok()?;
    let data = match &record.data {
        RecordData::A(address) => RData::A(A(*address)),
        RecordData::Aaaa(address) => RData::AAAA(AAAA(*address)),
        RecordData::Cname(target) => RData::CNAME(CNAME(fqdn(target).ok()?)),
        RecordData::Ns(target) => RData::NS(NS(fqdn(target).ok()?)),
        RecordData::Txt(text) => RData::TXT(TXT::new(vec![text.clone()])),
        RecordData::Soa {
            primary,
            mailbox,
            serial,
            minimum,
        } => RData::SOA(SOA::new(
            fqdn(primary).ok()?,
            fqdn(mailbox).ok()?,
            *serial,
            REFRESH_SECONDS,
            RETRY_SECONDS,
            EXPIRE_SECONDS,
            *minimum,
        )),
    };
    Some(Record::from_rdata(name, record.ttl, data))
}

fn fqdn(name: &str) -> Result<Name, hickory_proto::ProtoError> {
    Name::from_ascii(format!("{}.", name.trim_end_matches('.')))
}

fn plain(name: &Name) -> String {
    name.to_ascii().trim_end_matches('.').to_ascii_lowercase()
}

fn kind_of(record_type: RecordType) -> RecordKind {
    match record_type {
        RecordType::A => RecordKind::A,
        RecordType::AAAA => RecordKind::Aaaa,
        RecordType::CNAME => RecordKind::Cname,
        RecordType::TXT => RecordKind::Txt,
        RecordType::SOA => RecordKind::Soa,
        RecordType::NS => RecordKind::Ns,
        RecordType::ANY => RecordKind::Any,
        RecordType::AXFR | RecordType::IXFR => RecordKind::Transfer,
        _ => RecordKind::Other,
    }
}

fn code_of(code: ReplyCode) -> ResponseCode {
    match code {
        ReplyCode::NoError => ResponseCode::NoError,
        ReplyCode::FormatError => ResponseCode::FormErr,
        ReplyCode::NameError => ResponseCode::NXDomain,
        ReplyCode::NotImplemented => ResponseCode::NotImp,
        ReplyCode::Refused => ResponseCode::Refused,
    }
}
