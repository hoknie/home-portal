use serde::{Deserialize, Serialize};

use crate::types::Tail;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredTail {
    pub tail: String,
    pub bytes: u64,
}

impl StoredTail {
    pub fn of(tail: &Tail) -> StoredTail {
        StoredTail {
            tail: tail.text(),
            bytes: tail.total,
        }
    }

    pub fn into_tail(self) -> Tail {
        Tail::restored(self.tail.as_bytes(), self.bytes)
    }
}
