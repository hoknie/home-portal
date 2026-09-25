use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reception {
    pub at: OffsetDateTime,
    pub status: u16,
}
