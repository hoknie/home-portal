use super::{ReplyCode, ResourceRecord};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    pub code: ReplyCode,
    pub authoritative: bool,
    pub answers: Vec<ResourceRecord>,
    pub authority: Vec<ResourceRecord>,
}

impl Reply {
    pub fn bare(code: ReplyCode) -> Reply {
        Reply {
            code,
            authoritative: false,
            answers: Vec::new(),
            authority: Vec::new(),
        }
    }
}
