use std::net::IpAddr;

use portal_model::Environment;

use super::answer;
use crate::codecs::{read, write};
use crate::types::{Reply, ReplyCode, ZoneBook};

pub const UDP_LIMIT: usize = 1232;

pub fn handle(
    book: &ZoneBook,
    peer: IpAddr,
    bytes: &[u8],
    limit: Option<usize>,
) -> Option<Vec<u8>> {
    respond(book, &book.environment_of(peer), bytes, limit)
}

pub fn respond(
    book: &ZoneBook,
    environment: &Environment,
    bytes: &[u8],
    limit: Option<usize>,
) -> Option<Vec<u8>> {
    let incoming = read(bytes)?;
    let reply = if environment.is_internet() {
        Reply::bare(ReplyCode::Refused)
    } else if !incoming.query {
        Reply::bare(ReplyCode::NotImplemented)
    } else {
        match &incoming.question {
            None => Reply::bare(ReplyCode::FormatError),
            Some(question) => answer(book, environment, question),
        }
    };
    write(&incoming, &reply, limit)
}
