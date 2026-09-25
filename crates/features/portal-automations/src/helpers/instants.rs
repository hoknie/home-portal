use jiff::Timestamp;
use time::OffsetDateTime;

pub fn offset_of(instant: Timestamp) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp_nanos(instant.as_nanosecond())
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
}
