use portal_model::Environment;

use crate::types::{
    Question, RecordData, RecordKind, Reply, ReplyCode, ResourceRecord, Zone, ZoneBook,
};

pub fn answer(book: &ZoneBook, environment: &Environment, question: &Question) -> Reply {
    if environment.is_internet() || matches!(question.kind, RecordKind::Any | RecordKind::Transfer)
    {
        return Reply::bare(ReplyCode::Refused);
    }
    let Some(zone) = book.zone_of(&question.name) else {
        return Reply::bare(ReplyCode::Refused);
    };
    let mut reply = Reply {
        code: ReplyCode::NoError,
        authoritative: true,
        answers: Vec::new(),
        authority: Vec::new(),
    };
    let at_apex = question.name == zone.apex;
    if at_apex && question.kind == RecordKind::Soa {
        reply.answers.push(start_of_authority(book, zone));
        return reply;
    }
    if at_apex && question.kind == RecordKind::Ns {
        reply.answers.push(record(
            book,
            &zone.apex,
            RecordData::Ns(book.nameserver_of(zone)),
        ));
        return reply;
    }
    let Some(records) = book.records(&question.name, environment) else {
        if !at_apex {
            reply.code = ReplyCode::NameError;
        }
        reply.authority.push(start_of_authority(book, zone));
        return reply;
    };
    let matching: Vec<&RecordData> = records
        .iter()
        .filter(|data| data.kind() == question.kind)
        .collect();
    let alias = records.iter().find_map(|data| match data {
        RecordData::Cname(target) => Some(target.clone()),
        _ => None,
    });
    if !matching.is_empty() {
        reply.answers = matching
            .into_iter()
            .map(|data| record(book, &question.name, data.clone()))
            .collect();
    } else if let Some(target) = alias {
        reply.answers.push(record(
            book,
            &question.name,
            RecordData::Cname(target.clone()),
        ));
        let followed = book.records(&target, environment).unwrap_or_default();
        reply.answers.extend(
            followed
                .iter()
                .filter(|data| data.kind() == question.kind)
                .map(|data| record(book, &target, data.clone())),
        );
    } else {
        reply.authority.push(start_of_authority(book, zone));
    }
    reply
}

fn record(book: &ZoneBook, name: &str, data: RecordData) -> ResourceRecord {
    ResourceRecord {
        name: name.to_string(),
        ttl: book.ttl,
        data,
    }
}

fn start_of_authority(book: &ZoneBook, zone: &Zone) -> ResourceRecord {
    record(
        book,
        &zone.apex,
        RecordData::Soa {
            primary: book.nameserver_of(zone),
            mailbox: format!("{}.{}", ZoneBook::MAILBOX_LABEL, zone.apex),
            serial: book.serial,
            minimum: book.ttl,
        },
    )
}
