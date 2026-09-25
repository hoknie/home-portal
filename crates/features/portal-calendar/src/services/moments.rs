use time::macros::format_description;
use time::{Date, OffsetDateTime, PrimitiveDateTime, UtcOffset};

pub const UTC_SUFFIX: char = 'Z';

pub fn offset_of(timezone: Option<&str>) -> Result<UtcOffset, String> {
    let Some(text) = timezone.map(str::trim).filter(|text| !text.is_empty()) else {
        return Ok(UtcOffset::UTC);
    };
    if text.eq_ignore_ascii_case("utc") || text.eq_ignore_ascii_case("z") {
        return Ok(UtcOffset::UTC);
    }
    let format = format_description!("[offset_hour sign:mandatory]:[offset_minute]");
    UtcOffset::parse(text, format).map_err(|_| {
        "must be UTC or an offset such as +03:00; named zones are not read by the portal"
            .to_string()
    })
}

pub fn parse_moment(value: &str, offset: UtcOffset) -> Option<(OffsetDateTime, bool)> {
    let value = value.trim();
    if value.len() == 8 {
        let date = format_description!("[year][month][day]");
        let parsed = Date::parse(value, date).ok()?;
        let moment = parsed.midnight().assume_offset(offset);
        return Some((moment, true));
    }
    let utc = value.ends_with(UTC_SUFFIX);
    let bare = value.trim_end_matches(UTC_SUFFIX);
    let stamp = format_description!("[year][month][day]T[hour][minute][second]");
    let parsed = PrimitiveDateTime::parse(bare, stamp).ok()?;
    let moment = if utc {
        parsed.assume_utc()
    } else {
        parsed.assume_offset(offset)
    };
    Some((moment, false))
}
