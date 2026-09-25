use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct Attempts {
    pub failures: u32,
    pub first_failure_at: OffsetDateTime,
    pub locked_until: Option<OffsetDateTime>,
}
