use jiff::Timestamp;
use jiff::civil::{Date, DateTime, Time};
use jiff::tz::{AmbiguousOffset, TimeZone};

use super::Wildcards;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schedule {
    pub minutes: u64,
    pub hours: u64,
    pub days: u64,
    pub months: u64,
    pub weekdays: u64,
    pub wildcards: Wildcards,
}

impl Schedule {
    pub const SEARCH_DAYS: i32 = 366 * 4 + 1;

    pub fn next_after(&self, after: Timestamp, zone: &TimeZone) -> Option<Timestamp> {
        let first = after.to_zoned(zone.clone()).date().yesterday().ok()?;
        let mut found: Option<Timestamp> = None;
        let mut day = first;
        for _ in 0..Self::SEARCH_DAYS {
            if let Some(best) = found {
                let next = self.earliest_on(day, after, zone);
                return Some(next.map_or(best, |next| next.min(best)));
            }
            found = self.earliest_on(day, after, zone);
            day = day.tomorrow().ok()?;
        }
        found
    }

    pub fn matches_day(&self, date: Date) -> bool {
        if !Self::has(self.months, date.month() as u8) {
            return false;
        }
        let by_day = Self::has(self.days, date.day() as u8);
        let by_weekday = Self::has(self.weekdays, date.weekday().to_sunday_zero_offset() as u8);
        match (self.wildcards.day, self.wildcards.weekday) {
            (false, false) => by_day || by_weekday,
            (false, true) => by_day,
            (true, false) => by_weekday,
            (true, true) => true,
        }
    }

    fn earliest_on(&self, date: Date, after: Timestamp, zone: &TimeZone) -> Option<Timestamp> {
        if !self.matches_day(date) {
            return None;
        }
        let mut best: Option<Timestamp> = None;
        for hour in (0..24u8).filter(|hour| Self::has(self.hours, *hour)) {
            for minute in (0..60u8).filter(|minute| Self::has(self.minutes, *minute)) {
                let time = Time::new(hour as i8, minute as i8, 0, 0).ok()?;
                for instant in self.instants_of(date.to_datetime(time), zone) {
                    if instant > after && best.is_none_or(|best| instant < best) {
                        best = Some(instant);
                    }
                }
            }
        }
        best
    }

    fn instants_of(&self, civil: DateTime, zone: &TimeZone) -> Vec<Timestamp> {
        let ambiguous = zone.to_ambiguous_timestamp(civil);
        match ambiguous.offset() {
            AmbiguousOffset::Unambiguous { offset } => {
                offset.to_timestamp(civil).ok().into_iter().collect()
            }
            AmbiguousOffset::Gap { after, .. } => after
                .to_timestamp(civil)
                .ok()
                .and_then(|early| zone.following(early).next())
                .map(|transition| transition.timestamp())
                .into_iter()
                .collect(),
            AmbiguousOffset::Fold { before, after } => {
                let first = before.to_timestamp(civil).ok();
                let second = after.to_timestamp(civil).ok();
                if self.wildcards.hour {
                    first.into_iter().chain(second).collect()
                } else {
                    first.into_iter().collect()
                }
            }
        }
    }

    pub fn has(set: u64, value: u8) -> bool {
        set >> value & 1 == 1
    }
}
