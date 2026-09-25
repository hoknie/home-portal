use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seen {
    pub started_at: OffsetDateTime,
    pub last_at: OffsetDateTime,
    pub count: u32,
}

impl Seen {
    pub fn once(at: OffsetDateTime) -> Seen {
        Seen {
            started_at: at,
            last_at: at,
            count: 1,
        }
    }
}
